class Codes < Formula
  desc "Tree-sitter based local code search CLI"
  homepage "https://github.com/4fuu/code-search-cli"
  license "MIT"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.0/codes-v0.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "11c616a17aa5f1f0da394d8f0c0efeac8af54c4642d3e7c842acce1e558bacc5" # macos_arm64
    else
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.0/codes-v0.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "43f87f1258e415a301cb8b27a0be36c50ae4c3e9e7652563caf54f002d175d2e" # macos_x86
    end
  end

  on_linux do
    url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.0/codes-v0.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "cf36e25023a9bf96863815c6bbbe9341edcc3247ec00987b9a548d26215a4c51" # linux_x86
  end

  def install
    bin.install "codes"
    doc.install "README.md", "README.zh-CN.md", "LICENSE"
  end

  test do
    assert_match "Tree-sitter based code search CLI", shell_output("#{bin}/codes --help")
  end
end
