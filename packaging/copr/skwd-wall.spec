%global appname skwd-wall

Name:           skwd-wall
Version:        0.1.0
Release:        1%{?dist}
Summary:        Quickshell-based wallpaper selector with color sorting and Matugen integration

License:        MIT
URL:            https://github.com/liixini/skwd-wall
Source0:        %{url}/archive/refs/heads/experimental/rust-refactor.tar.gz#/%{name}-rust-refactor.tar.gz

BuildArch:      noarch

Requires:       skwd-daemon
Requires:       quickshell
Requires:       qt6-qtmultimedia
Requires:       qt6-qtdeclarative
Requires:       qt6-qtimageformats
Requires:       awww
Requires:       matugen
Requires:       curl
Requires:       sqlite
Requires:       ffmpeg-free
Requires:       ImageMagick
Requires:       inotify-tools
Requires:       google-roboto-fonts
Requires:       google-roboto-condensed-fonts
Requires:       google-roboto-mono-fonts
Requires:       skwd-fonts

Requires:       mpvpaper
Requires:       jq

Recommends:     ollama

%description
A Quickshell-based image, video, and Wallpaper Engine wallpaper selector with
color sorting, Matugen integration, tag system, and Wallhaven & Steam in-app
browsing. Features three visual presentation styles with rich animations.

%prep
%autosetup -n %{name}-experimental-rust-refactor

%build
# Nothing to build - QML application

%install
install -dm 0755 %{buildroot}%{_datadir}/%{appname}
cp -a shell.qml qml/ %{buildroot}%{_datadir}/%{appname}/

install -dm 0755 %{buildroot}%{_datadir}/%{appname}/data
cp -a data/matugen/ %{buildroot}%{_datadir}/%{appname}/data/
cp -a data/scripts/ %{buildroot}%{_datadir}/%{appname}/data/
install -Dpm 0644 data/config.json.example %{buildroot}%{_datadir}/%{appname}/data/config.json.example

install -Dpm 0644 data/skwd-wall.desktop %{buildroot}%{_datadir}/applications/skwd-wall.desktop
install -Dpm 0644 LICENSE %{buildroot}%{_datadir}/licenses/%{name}/LICENSE

%post
echo "skwd-wall installed."
echo "Make sure skwd-daemon is running:"
echo "  systemctl --user enable --now skwd-daemon.service"
echo "Launch the wallpaper selector with:"
echo "  skwd wall toggle"

%files
%license LICENSE
%{_datadir}/%{appname}/
%{_datadir}/applications/skwd-wall.desktop
