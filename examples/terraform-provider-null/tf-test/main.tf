terraform {
  required_providers {
    null = {
      source  = "localhost/aneoconsulting/null"
      version = "= 0.1.0"
    }
  }
}

resource "null_resource" "test" {
  triggers = {
    a = 1
  }
}
