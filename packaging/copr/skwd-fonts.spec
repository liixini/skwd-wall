%global fontname skwd-fonts

Name:           skwd-fonts
Version:        1.0.0
Release:        1%{?dist}
Summary:        Icon fonts required by Skwd-wall (Nerd Fonts Symbols + Material Design Icons)

License:        MIT AND Apache-2.0
URL:            https://github.com/liixini/skwd-wall

Source0:        https://github.com/ryanoasis/nerd-fonts/raw/HEAD/patched-fonts/NerdFontsSymbolsOnly/SymbolsNerdFontMono-Regular.ttf
Source1:        https://github.com/Templarian/MaterialDesign-Webfont/raw/master/fonts/materialdesignicons-webfont.ttf

BuildArch:      noarch

Requires:       fontconfig

%description
Bundles Nerd Fonts Symbols Only and Material Design Icons Desktop fonts
required by Skwd-wall for UI icons and symbols.

%install
install -Dpm 0644 %{SOURCE0} %{buildroot}%{_datadir}/fonts/%{fontname}/SymbolsNerdFontMono-Regular.ttf
install -Dpm 0644 %{SOURCE1} %{buildroot}%{_datadir}/fonts/%{fontname}/materialdesignicons-webfont.ttf

%post
fc-cache -f %{_datadir}/fonts/%{fontname} 2>/dev/null || :

%postun
if [ $1 -eq 0 ]; then
  fc-cache -f %{_datadir}/fonts/%{fontname} 2>/dev/null || :
fi

%files
%dir %{_datadir}/fonts/%{fontname}
%{_datadir}/fonts/%{fontname}/SymbolsNerdFontMono-Regular.ttf
%{_datadir}/fonts/%{fontname}/materialdesignicons-webfont.ttf
