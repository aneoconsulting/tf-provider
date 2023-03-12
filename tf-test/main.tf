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

  dynamic "read" {
    for_each = null_resource.pouet.id != null ? {} : {}
    labels   = ["pouet"]
    content {
      cmd = "echo pouet"
    }
  }

  update {
    cmd      = "echo update"
    triggers = null_resource.pouet.id != null ? {} : {}
  }

  read "pouet" {
    cmd = null_resource.pouet.id
  }
}
