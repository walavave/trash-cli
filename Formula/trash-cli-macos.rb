class TrashCliMacos < Formula
  desc "Native macOS trash CLI implemented in Rust"
  homepage "https://github.com/OWNER/REPO"
  url "https://github.com/OWNER/REPO/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_TARBALL_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "trash-cli-macos")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/trash --version")
  end
end
