/// Infrastructure client for interacting with the BoardGameGeek (BGG) API.
///
/// This module provides the necessary logic to fetch, parse, and map
/// external board game data into the local domain entities. It serves
/// as an anti-corruption layer between the BGG XML schema and the
/// internal `Game` models.
pub mod bgg_client;
