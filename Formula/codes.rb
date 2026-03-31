class Codes < Formula
  desc "Tree-sitter based local code search CLI"
  homepage "https://github.com/4fuu/code-search-cli"
  license "MIT"
  version "0.1.3"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.3/codes-v0.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "679079e66260e89558cc6c4331bc32ab0c16b3a24f70505d03a6a5f57d4517a4" # macos_arm64
    else
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.3/codes-v0.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "a8a7bd78ff8d245517390909bad1ec59af78f5cc59b8b23ef620cb0fc88c4507" # macos_x86
    end
  end

  on_linux do
    url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.3/codes-v0.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "b498129c3b4487e8c3d8e5dba3ad421d64499b7f2d784e15ff8d081aae5cc27d" # linux_x86
  end

  def install
    bin.install "codes"
    doc.install "README.md", "README.zh-CN.md", "LICENSE"
  end

  test do
    assert_match "Tree-sitter based code search CLI", shell_output("#{bin}/codes --help")
  end
end
