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
  read "a" {
    cmd = "sleep 1; echo -n a"
  }
  read "b" {
    cmd = "sleep 1; echo -n b"
  }
  read "c" {
    cmd = "sleep 1; echo -n c"
  }
  read "d" {
    cmd = "sleep 1; echo -n d"
  }
  read "e" {
    cmd = "sleep 1; echo -n e"
  }
  read "f" {
    cmd = "sleep 1; echo -n f"
  }
  read "g" {
    cmd = "sleep 1; echo -n g"
  }
  read "h" {
    cmd = "sleep 1; echo -n h"
  }
  read "i" {
    cmd = "sleep 1; echo -n i"
  }
  read "j" {
    cmd = "sleep 1; echo -n j"
  }
  read "k" {
    cmd = "sleep 1; echo -n k"
  }
  read "l" {
    cmd = "sleep 1; echo -n l"
  }
  read "m" {
    cmd = "sleep 1; echo -n m"
  }
  read "n" {
    cmd = "sleep 1; echo -n n"
  }
  read "o" {
    cmd = "sleep 1; echo -n o"
  }
  read "p" {
    cmd = "sleep 1; echo -n p"
  }
  read "q" {
    cmd = "sleep 1; echo -n q"
  }
  read "r" {
    cmd = "sleep 1; echo -n r"
  }
  read "s" {
    cmd = "sleep 1; echo -n s"
  }
  read "t" {
    cmd = "sleep 1; echo -n t"
  }
  read "u" {
    cmd = "sleep 1; echo -n u"
  }
  read "v" {
    cmd = "sleep 1; echo -n v"
  }
  read "w" {
    cmd = "sleep 1; echo -n w"
  }
  read "x" {
    cmd = "sleep 1; echo -n x"
  }
  read "y" {
    cmd = "sleep 1; echo -n y"
  }
  read "z" {
    cmd = "sleep 1; echo -n z"
  }
}

data "cmd_local_exec" "pouet" {
  inputs = {
    a = cmd_ssh_exec.test.state.a
  }

  read "a" {
    cmd = "echo -n a"
  }
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
