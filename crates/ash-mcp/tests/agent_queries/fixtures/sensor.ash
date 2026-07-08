interface Temperature {
  read(String) -> String
}

interface Humidity {
  read(String) -> String
}

fn read_sensor(kind: String, id: String) -> String {
  "value"
}
