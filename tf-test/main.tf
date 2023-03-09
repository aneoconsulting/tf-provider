terraform {
  required_providers {
    cmd = {
      source  = "lemaitre.re/lemaitre/cmd"
      version = ">= 0.1.0"
    }
  }
}

provider "cmd" {
}

resource "null_resource" "pouet" {

}

resource "cmd_test" "test" {
  dummy  = 3
  dummy2 = null_resource.pouet.id
  read "pouet" {}
}
