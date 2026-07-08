interface Reader {
  read(String) -> String
}

fn helper(x: Int) -> Int {
  x + 1
}

interface Sensor {
  read(String) -> String
}
