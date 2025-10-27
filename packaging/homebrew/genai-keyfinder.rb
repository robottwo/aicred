class GenaiKeyfinder < Formula
  desc "Cross-platform GenAI key discovery tool"
  homepage "https://github.com/robottwo/aicred"
  url "https://github.com/robottwo/aicred/archive/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA"
  license "MIT"
  version "0.1.0"

  depends_on "rust" => :build
  depends_on "pkg-config" => :build
  depends_on "openssl@3"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/keyfinder --version")
  end
end