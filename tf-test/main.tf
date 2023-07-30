terraform {
  required_providers {
    generic = {
      source  = "localhost/lemaitre/generic"
      version = "= 0.1.1"
    }
  }
}

provider "generic" {
}

resource "null_resource" "pouet" {

}

resource "generic_ssh_cmd" "test" {
  connect {
    host    = "10.42.0.2"
    user    = "dummy-user"
    keyfile = "dummy.ed25519"
  }
  inputs = {
    a = null_resource.pouet.id
  }

  create {
    cmd = "env | grep -P 'INPUT|STATE|HOME|ID|VERSION'"
  }
  destroy {
    cmd = "env | grep -P 'INPUT|STATE|HOME|ID|VERSION'"
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
    cmd      = "env | grep -P 'INPUT|STATE|HOME|ID|VERSION'"
    reloads  = ["plop"]
  }

  read "plop" {
    cmd       = "false"
    faillible = true
  }
  read "pouet" {
    cmd = "echo -n pouet"
  }
  read "resolv" {
    cmd = "cat resolv.conf"
    dir = "/etc"
  }
}

data "generic_local_cmd" "pouet" {
  inputs = {
    a = generic_ssh_cmd.test.state.pouet
  }

  read "a" {
    cmd = "cat resolv.conf"
    dir = "/etc"
  }
}

data "generic_ssh_file" "pouet" {
  connect {
    host    = "10.42.0.2"
    user    = "dummy-user"
    keyfile = "dummy.ed25519"
  }
  path = "/etc/resolv.conf"
}

resource "generic_ssh_file" "plop" {
  connect {
    host    = "10.42.0.2"
    user    = "dummy-user"
    keyfile = "dummy.ed25519"
  }
  path           = "plop.txt"
  content_source = "client.crt"
  overwrite      = true
}

output "exec" {
  value = {
    inputs  = generic_ssh_cmd.test.inputs
    outputs = generic_ssh_cmd.test.state
  }
}
output "data_cmd" {
  value = {
    inputs  = data.generic_local_cmd.pouet.inputs
    outputs = data.generic_local_cmd.pouet.outputs
  }
}

output "datafile" {
  value = data.generic_ssh_file.pouet
}
output "file" {
  value = generic_ssh_file.plop
}
