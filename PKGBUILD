# Maintainer: Gaurav Atreya <allmanpride at gmail dotcom>
pkgname=nam
pkgver=0.1
pkgrel=1
pkgdesc="Tool to run commands based on a templates"
arch=('any')
url="https://github.com/Atreyagaurav/nam"
license=('GPL3')
depends=('gcc-libs')
makedepends=('rust' 'cargo' 'git')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/Atreyagaurav/nam/archive/refs/tags/v${pkgver}.tar.gz")
md5sums=('SKIP')

build() {
	cd "$srcdir/${pkgname}-${pkgver}"
	cargo build --release
}

package() {
    cd "$srcdir/${pkgname}-${pkgver}"
    mkdir -p "$pkgdir/usr/bin"
    cp target/release/nam "$pkgdir/usr/bin/nam"
}
