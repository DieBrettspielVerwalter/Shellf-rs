{
  version: "0.7.0",
  test_runs: [
    {
      name: $test_name,
      date: now | todateiso8601,
      "nr-of-tests": 1,
      data: {},
      logs: $logs,
      tests: [
        {
          name: $test_name,
          filepath: "",
          line: 0,
          state: "passed",
          "covered-files": (
            .data[0].files
            | map(
                . as $file
                | (
                  if .filename | startswith($pwd) then .filename 
                    | ltrimstr($pwd) else .filename end) as $rel_filename
                | {
                filepath: $rel_filename,
                # Filtert die globalen Traces auf die Traces, die zu DIESER Datei gehören
                "covered-traces": (
                  # Hier stellen wir sicher, dass wir nur Objekte aus $traces mappen
                  $traces 
                  # | map(select($file.filename | endswith(.file)))
                  # | map({"file": ($file | endswith(.file))})
                  # | map({"file": .})
                  # | map(select(type == "object" and (.file | type) == "string")) 
                  # | map({
                  #     "file_match": {
                  #       "filename": ($file.filename | type),
                  #       "file": (.file | type)
                  #     }
                  #   })
                  # | select($file.filename | endswith(.file? // "!?!"))
                  # | map(
                  #   select(type == "object" and (.file | type) == "string") 
                  #   | select($file.filename | endswith(.file))
                  # )
                  | map(. as $trace | {"raw":$trace, "file": $file.filename, "cmp": ($file.filename | endswith($trace.file))}) | map(select(.cmp))
                  | map({
                      "req-ids": [.raw.req_id],
                      line: (.raw.line | tonumber)
                    })
                ),
                "covered-lines": (
                    .segments
                    | map({line: .[0], hits: .[2]})
                    | map(select(.hits > 0))
                )
            })
          )
        }
      ]
    }
  ]
}