interface Reader {
  read(String) -> String
}

fn helper(x: Int) -> Int {
  x + 1
}

capability sensor: epistemic(id: String)
