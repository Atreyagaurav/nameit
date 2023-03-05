# Maintainer: Gaurav Atreya <allmanpride at gmail dotcom>
pkgname=nameit
pkgver=0.1
pkgrel=1
pkgdesc="Tool to run commands based on a templates"
arch=('any')
url="https://github.com/Atreyagaurav/${pkgname}"
license=('GPL3')
depends=('gcc-libs')
makedepends=('rust' 'cargo' 'git')
source=("${pkgname}::git+https://github.com/Atreyagaurav/${pkgname}.git")
md5sums=('SKIP')

pkgver() {
    cd "$srcdir/${pkgname}"
    printf "%s" "$(git tag -l | head -n1 | sed 's/\([^-]*-\)g/r\1/;s/-/./g')"
}

prepare() {
    cd "$srcdir/${pkgname}"
    git checkout "tags/$(git tag -l | head -n1)"
}


build() {
	cd "$srcdir/${pkgname}"
	cargo build --release
}

package() {
    cd "$srcdir/${pkgname}"
    mkdir -p "$pkgdir/usr/bin"
    cp "target/release/${pkgname}" "$pkgdir/usr/bin/${pkgname}"
}
