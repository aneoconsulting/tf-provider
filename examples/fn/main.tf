terraform {
  required_version = ">= 1.8.0"
  required_providers {
    fn = {
      source  = "localhost/aneoconsulting/fn"
      version = "= 0.1.0"
    }
  }
}

output "five" {
  value = provider::fn::add(2, 3)
}
