class Codes < Formula
  desc "Tree-sitter based local code search CLI"
  homepage "https://github.com/4fuu/code-search-cli"
  license "MIT"
  version "0.1.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.1/codes-v0.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "737b89c94482d8dc8aa1b3c019b0b8eaa0a2d77658063c34a8c1da2de1823dde" # macos_arm64
    else
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.1/codes-v0.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "cac697ede09f490c080e80f114b24aa8175d9576770f5deebb2b656fa7969425" # macos_x86
    end
  end

  on_linux do
    url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.1/codes-v0.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "796aac5e1af56602f42a3468034c7c5d0c6f22db9674979d52e1dde2bb8cbe26" # linux_x86
  end

  def install
    bin.install "codes"
    doc.install "README.md", "README.zh-CN.md", "LICENSE"
  end

  test do
    assert_match "Tree-sitter based code search CLI", shell_output("#{bin}/codes --help")
  end
end
