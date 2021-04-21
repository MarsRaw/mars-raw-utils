%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: mars_raw_utils
Summary: Utilities for working with publicly available raw MSL, Mars2020, and InSight images
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: MIT
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_MCZ_LEFT_INPAINT_MASK_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_MCZ_RIGHT_INPAINT_MASK_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_SCAM_FLAT_Sol1_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_SCAM_MASK_Sol1_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_WATSON_FLAT_V0.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_WATSON_INPAINT_MASK_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_FLB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_FRB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_MAHLI_FLAT_Sol2904_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_MAHLI_INPAINT_Sol2904_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_MCAM_LEFT_INPAINT_Sol3082_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_NLB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_NRB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_NRB_INPAINT_Sol3052_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_RLB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_RRB_FLAT_V1.png