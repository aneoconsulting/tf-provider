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

resource "cmd_local_exec" "test" {
  inputs = {
    pouet = null_resource.pouet.id
  }

  create {
    cmd = "env | grep -P 'INPUT|STATE|HOME'"
  }
  destroy {
    cmd = "env | grep -P 'INPUT|STATE|HOME'"
  }

  read "plop" {
    cmd = "env | grep -P 'INPUT|STATE|HOME'"
  }
}

output "exec" {
  value = cmd_local_exec.test
}
