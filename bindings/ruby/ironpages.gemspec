# frozen_string_literal: true

require_relative "lib/ironpages/version"

Gem::Specification.new do |spec|
  spec.name = "ironpages"
  spec.version = Ironpages::VERSION
  spec.authors = ["Paul Gaston Gouron", "Peilin Wu"]
  spec.summary = "Pure Rust HTML/CSS/Markdown to PDF converter"
  spec.description = <<~DESCRIPTION.strip
    Convert HTML, CSS, and Markdown to PDF with no browser or system
    dependencies. Native Rust extension for maximum performance.
  DESCRIPTION
  spec.homepage = "https://github.com/dickwu/ironpages"
  spec.metadata["documentation_uri"] = "https://dickwu.github.io/ironpages/get-started/ruby/"
  spec.metadata["source_code_uri"] = "https://github.com/dickwu/ironpages"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.1"

  spec.files = Dir[
    "lib/**/*.rb",
    "ext/**/*.{rb,rs,toml}",
    "README.md"
  ]
  spec.extensions = ["ext/ironpages/extconf.rb"]
  spec.require_paths = ["lib"]

  spec.add_dependency "rb_sys", "~> 0.9"
end
