class Codes < Formula
  desc "Tree-sitter based local code search CLI"
  homepage "https://github.com/4fuu/code-search-cli"
  license "MIT"
  version "0.1.2"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.2/codes-v0.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "28e356112a03ee374b47175a332f33efaf36745acb347b1fb346c3f37e04e17a" # macos_arm64
    else
      url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.2/codes-v0.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "8c1cc09ac6a9527cb976287a08c4293a08d3dce3b36646c8c65538b0aaed1944" # macos_x86
    end
  end

  on_linux do
    url "https://github.com/4fuu/code-search-cli/releases/download/v0.1.2/codes-v0.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "cfacbef68d2d12a56efcfd8ee691fa48fa17c7bffe37cbfb2c9293a2729e2ef6" # linux_x86
  end

  def install
    bin.install "codes"
    doc.install "README.md", "README.zh-CN.md", "LICENSE"
  end

  test do
    assert_match "Tree-sitter based code search CLI", shell_output("#{bin}/codes --help")
  end
end
