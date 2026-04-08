use anyhow::{anyhow, Result};
use quick_xml::de::from_str;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

/// Represents the response from the BGG search endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename = "items")]
struct BggSearchResponse {
    #[serde(rename = "item", default)]
    items: Vec<BggSearchItem>,
}

/// A single search result item from the BGG search API.
#[derive(Debug, Deserialize, Clone)]
pub struct BggSearchItem {
    /// The unique BGG internal ID for the board game.
    #[serde(rename = "@id")]
    pub id: String,
    /// The primary name of the game.
    #[serde(rename = "name")]
    pub name: BggValue,
    /// The published year, if provided by the API.
    #[serde(rename = "yearpublished")]
    pub year: Option<BggValue>,
}

/// Represents the response from the BGG "thing" endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename = "items")]
struct BggThingResponse {
    #[serde(rename = "item")]
    item: BggThingItem,
}

/// Represents a single game object returned by the BGG "thing" endpoint.
#[derive(Debug, Deserialize)]
struct BggThingItem {
    /// Optional statistics for the game, including ratings.
    #[serde(rename = "statistics")]
    statistics: Option<BggStatistics>,
    /// Links representing designers, categories, publishers, etc.
    #[serde(rename = "link")]
    links: Vec<BggLink>,
}

/// Represents game rating statistics.
#[derive(Debug, Deserialize)]
struct BggStatistics {
    /// Container for rating values.
    #[serde(rename = "ratings")]
    ratings: BggRatings,
}

/// Represents average ratings for a game.
#[derive(Debug, Deserialize)]
struct BggRatings {
    /// The average rating value.
    #[serde(rename = "average")]
    average: BggValue,
}

/// Represents a link associated with a BGG game, such as designer or category.
#[derive(Debug, Deserialize)]
struct BggLink {
    #[serde(rename = "@type")]
    link_type: String,
    #[serde(rename = "@value")]
    value: String,
}

/// A wrapper for values extracted from BGG's XML attribute-based schema.
#[derive(Debug, Deserialize, Clone)]
pub struct BggValue {
    /// The actual string value within the XML attribute.
    #[serde(rename = "@value")]
    pub value: String,
}

/// Aggregated details for a board game retrieved from the BGG "thing" API.
pub struct BggDetails {
    /// The community-average rating (0.0 - 10.0).
    pub rating: f32,
    /// A list of designers/authors.
    pub authors: Vec<String>,
    /// The primary publisher name.
    pub publisher: Option<String>,
    /// Categories assigned to the game (e.g., "Economic", "Fantasy").
    pub categories: Vec<String>,
}

/// An infrastructure client for interacting with the BoardGameGeek (BGG) XML API2.
///
/// This client provides methods to search for titles and fetch detailed metadata
/// for specific board games.
pub struct BggClient {
    api_key: String,
}

impl BggClient {
    /// Creates a new instance of the `BggClient`.
    ///
    /// # Arguments
    ///
    /// * `api_key` - A bearer token or API key used for authorized requests.
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    /// Searches for board games by name using the BGG search endpoint.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string (e.g., "Catan").
    ///
    /// # Returns
    ///
    /// A `Result` containing a list of [`BggSearchItem`] matches.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or the XML response
    /// cannot be parsed.
    pub async fn search(&self, query: &str) -> Result<Vec<BggSearchItem>> {
        let url = format!(
            "https://boardgamegeek.com/xmlapi2/search?query={}&type=boardgame",
            query
        );
        let xml = self.call_bgg(&url).await?;
        let res: BggSearchResponse = from_str(&xml)?;
        Ok(res.items)
    }

    /// Fetches extended metadata for a specific game ID.
    ///
    /// This method extracts ratings, authors, publishers, and categories
    /// from the BGG "thing" endpoint.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique BGG ID of the game.
    ///
    /// # Returns
    ///
    /// A `Result` containing the populated [`BggDetails`].
    ///
    /// # Errors
    ///
    /// Returns an error if the ID is invalid or the API returns an unexpected format.
    pub async fn get_details(&self, id: &str) -> Result<BggDetails> {
        let url = format!("https://boardgamegeek.com/xmlapi2/thing?id={}&stats=1", id);
        let xml = self.call_bgg(&url).await?;
        let res: BggThingResponse = from_str(&xml)?;

        // Extract average rating, defaulting to 0.0 if unavailable
        let rating = res
            .item
            .statistics
            .map(|s| s.ratings.average.value.parse().unwrap_or(0.0))
            .unwrap_or(0.0);

        let mut authors = Vec::new();
        let mut categories = Vec::new();
        let mut publisher = None;

        // Iterate over links to categorize designers, publishers, and game categories
        for link in res.item.links {
            match link.link_type.as_str() {
                "boardgamedesigner" => authors.push(link.value),
                "boardgamecategory" => categories.push(link.value),
                "boardgamepublisher" if publisher.is_none() => publisher = Some(link.value),
                _ => {}
            }
        }

        Ok(BggDetails {
            rating,
            authors,
            publisher,
            categories,
        })
    }

    /// Internal helper to perform HTTP GET requests to the BGG API.
    ///
    /// Sets required headers and handles non-success HTTP status codes.
    ///
    /// # Arguments
    ///
    /// * `url` - The full URL to query the BGG API.
    ///
    /// # Returns
    ///
    /// A `Result` containing the raw XML response as a `String`.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the status is non-successful,
    /// or the response is empty.
    async fn call_bgg(&self, url: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .user_agent("ShellfGameManager/1.0")
            .build()?;

        let response = client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("BGG API Fehler: {}", response.status()));
        }

        let text = response.text().await?;
        if text.trim().is_empty() {
            return Err(anyhow!("BGG API lieferte eine leere Antwort."));
        }

        Ok(text)
    }
}
