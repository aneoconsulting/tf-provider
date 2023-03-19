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
    a = null_resource.pouet.id
  }

  create {
    cmd = "env | grep -P 'INPUT|STATE|HOME'"
  }
  destroy {
    cmd = "env | grep -P 'INPUT|STATE|HOME'"
  }

  update {
    triggers = ["a"]
    cmd      = "echo update a"
    reloads  = ["plop"]
  }

  read "plop" {
    cmd = "echo -n plop"
  }
}

output "exec" {
  value = {
    inputs  = cmd_local_exec.test.inputs
    outputs = cmd_local_exec.test.state
  }
}
