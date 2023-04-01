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

resource "cmd_ssh_exec" "test" {
  connect {
    host    = "10.42.0.2"
    user    = "dummy-user"
    keyfile = "dummy.ed25519"
  }
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
    triggers = ["a", "b"]
    cmd      = "echo update a b"
    reloads  = ["plop"]
  }
  update {
    triggers = ["b", "c"]
    cmd      = "echo update b c"
    reloads  = ["plop"]
  }
  update {
    triggers = ["b", "d"]
    cmd      = "echo update b d"
    reloads  = ["plop"]
  }
  update {
    triggers = ["b"]
    cmd      = "echo update b"
    reloads  = ["plop"]
  }

  read "plop" {
    cmd = "echo -n plop"
  }
  read "pouet" {
    cmd = "echo -n pouet"
  }
}

data "cmd_local_exec" "pouet" {
  inputs = {
    a = cmd_ssh_exec.test.state.pouet
  }

  read "a" {
    cmd = "echo -n a"
  }
}

data "cmd_local_file" "pouet" {
  path = "a"
}

resource "cmd_local_file" "plop" {
  path           = "plop.txt"
  content_source = "client.crt"
  overwrite      = true
}

output "exec" {
  value = {
    inputs  = cmd_ssh_exec.test.inputs
    outputs = cmd_ssh_exec.test.state
  }
}
output "data_exec" {
  value = {
    inputs  = data.cmd_local_exec.pouet.inputs
    outputs = data.cmd_local_exec.pouet.outputs
  }
}

output "file" {
  value = cmd_local_file.plop
}
