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
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_SCAM_FLAT_RGB_Sol77_V2.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_SCAM_MASK_Sol1_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_WATSON_FLAT_V0.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_FLAT_SN_5001.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_WATSON_INPAINT_MASK_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_WATSON_PREFLIGHT_FLAT_RGB_FOCUS_11958_V1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_FLB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_FRB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_MAHLI_FLAT_Sol2904_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_MAHLI_INPAINT_Sol2904_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_MCAM_LEFT_INPAINT_Sol3082_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_MCAM_RIGHT_INPAINT_Sol3101_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_NLB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_NRB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_NRB_INPAINT_Sol3052_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_RLB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_RRB_FLAT_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/FLAT_ML_filter0_0-rjcal.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/FLAT_MR_filter0_0-rjcal.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/FLAT_MD_0_RGB_V1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_HELI_NAV_FLAT_Sol76_V1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/M20_HELI_RTE_FLAT_V3.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_CCAM_FLAT_Sol32_V1.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/MSL_CCAM_MASK_Sol3122_V2.png
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/NSYT_FLAT_SN_0203.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/NSYT_FLAT_SN_0210.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/caldata.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/msl_mahli_bay.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/msl_mahli_cwb.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/msl_mahli_ilt.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/msl_mahli_rad.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/msl_mcam_bay.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/msl_mcam_ilt.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/msl_mcam_rad.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_hrte_rad.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_watson_bay.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_watson_ilt.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_watson_rad.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_zcam_bay.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_zcam_ilt.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_zcam_rad.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_zcam_cwb.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_zcam_cb2.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_ncam_ilt.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20_ncam_rad.toml
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/L0_-motorcount-_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/L0_0000_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/L0_2448_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/L0_3834_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/L0_5194_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/L0_6720_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/L0_8652_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/L0_9600_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/R0_-motorcount-_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/R0_0000_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/R0_2448_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/R0_3834_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/R0_5194_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/R0_6720_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/R0_8652_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/R0_9600_cont_v1.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/meda_rad_flat_padded.jpg
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/msl/ilut/DECOMPAND0.TXT
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20/ilut/M20_MCZ_LUT2.txt
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20/ilut/M20_SI_LUT0.txt
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/m20/ilut/M20_LUT2_v2a.txt
%attr(0644,root,root) %config(noreplace) /usr/share/mars_raw_utils/data/nsyt/ilut/NSYT_LUT0.txt