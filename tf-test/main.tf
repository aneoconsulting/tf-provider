terraform {
  required_providers {
    cmd = {
      source  = "lemaitre.re/lemaitre/cmd"
      version = ">= 0.1.0"
    }
  }
}

provider "cmd" {
    foo = "bar"
}

resource "cmd_thing" "test" {
    bar = "baz"
}
