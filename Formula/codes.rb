class Codes < Formula
  desc "Tree-sitter based local code search CLI"
  homepage "https://github.com/4fuu/code-search-cli"
  license "MIT"
  version "0.1.4"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.4/codes-v0.1.4-aarch64-apple-darwin.tar.gz"
      sha256 "c741735b87b7b7d92b51e565e83d61ad78d7558209c9171b885f9761211bbc8b" # macos_arm64
    else
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.4/codes-v0.1.4-x86_64-apple-darwin.tar.gz"
      sha256 "7c07de5b2b5afc93486df1f0dbac59bf54156a4de899da2d8316f34ddca5a718" # macos_x86
    end
  end

  on_linux do
    url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.4/codes-v0.1.4-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "977cab9d4172e9b39dced6b961b60f60b9107742a94fbf7c9b959e54241ed588" # linux_x86
  end

  def install
    bin.install "codes"
    doc.install "README.md", "README.zh-CN.md", "LICENSE"
  end

  test do
    assert_match "Tree-sitter based code search CLI", shell_output("#{bin}/codes --help")
  end
end
