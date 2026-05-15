class Codes < Formula
  desc "Tree-sitter based local code search CLI"
  homepage "https://github.com/4fuu/code-search-cli"
  license "MIT"
  version "0.1.4"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.4/codes-v0.1.4-aarch64-apple-darwin.tar.gz"
      sha256 "7a6d5734d6664c351fa001a120fc7ad362b3a767ae51dcf144e09d36709f71e5" # macos_arm64
    else
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.4/codes-v0.1.4-x86_64-apple-darwin.tar.gz"
      sha256 "b10b7adc36f7f0525623bec04b151c099e5537a87767914537536956d337d587" # macos_x86
    end
  end

  on_linux do
    url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.4/codes-v0.1.4-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "6ca852c596b3e4b0c54d7d80207d66494477ee59ab6d120fd32a13e9c42b7848" # linux_x86
  end

  def install
    bin.install "codes"
    doc.install "README.md", "README.zh-CN.md", "LICENSE"
  end

  test do
    assert_match "Tree-sitter based code search CLI", shell_output("#{bin}/codes --help")
  end
end
