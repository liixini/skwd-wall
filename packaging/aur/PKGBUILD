# Maintainer: liixini <https://github.com/liixini>
pkgname=skwd-wall
pkgver=r30.a8d3082
pkgrel=1
pkgdesc='Quickshell-based image, video & wallpaper engine wallpaper selector with color sorting, Matugen integration, and Wallhaven & Steam in-app browsing'
arch=('any')
url='https://github.com/liixini/skwd-wall'
license=('MIT')
depends=(
  'quickshell'
  'qt6-multimedia'
  'qt6-declarative'
  'curl'
  'sqlite'
  'ffmpeg'
  'imagemagick'
  'inotify-tools'
  'matugen'
  'ttf-nerd-fonts-symbols'
  'ttf-roboto'
  'jq'
)
optdepends=(
  'awww: static wallpaper application on non-KDE Wayland'
  'mpvpaper: video/gif wallpaper on non-KDE Wayland'
  'ollama: Local LLM for automated tagging'
  'steamcmd: Steam Workshop Wallpaper Engine downloads'
  'linux-wallpaperengine: Wallpaper Engine scene support'
)
install="${pkgname}.install"
source=("${pkgname}::git+https://github.com/liixini/skwd-wall.git")
sha256sums=('SKIP')

pkgver() {
  cd "$pkgname"
  printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

package() {
  cd "$pkgname"

  local _sharedir="$pkgdir/usr/share/skwd-wall"

  install -dm755 "$_sharedir"
  cp -a shell.qml daemon.qml qml/ "$_sharedir/"

  install -dm755 "$_sharedir/data"
  cp -a data/matugen/ "$_sharedir/data/"
  cp -a data/scripts/ "$_sharedir/data/"
  install -Dm644 data/config.json.example "$_sharedir/data/config.json.example"

  install -Dm644 data/skwd-wall.desktop "$pkgdir/usr/share/applications/skwd-wall.desktop"
  install -Dm644 data/skwd-wall.service "$pkgdir/usr/lib/systemd/user/skwd-wall.service"

  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
