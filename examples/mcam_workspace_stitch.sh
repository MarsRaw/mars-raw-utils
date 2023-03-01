#!/bin/bash

# Usage: mcam_workspace_stitch.sh <sol> <image1> <image2> <image3> \
#              <image4> <image5> <image6>
#
# This is a simple script which stitches together a 3x2 MastCam 
# workspace mosaic. It takes the sol and MRU-calibrated images 
# as inputs and uses Hugin for the stitching. This does not 
# support workspace extension sequences. 
#
# Several assumptions are made here which may not work perfectly. 

SOL=$1
FILE1=$2
FILE2=$3
FILE3=$4
FILE4=$5
FILE5=$6
FILE6=$7

PREFIX=MSL_MCAM_${SOL}_WORKSPACE

cat > ${PREFIX}.pto << EOF
# hugin project file
#hugin_ptoversion 2
p f2 w4308 h3015 v50  k0 E0.643533 R0 S14,3942,22,2829 n"TIFF_m c:LZW r:CROP"
m i0

# image lines
#-hugin  cropFactor=1
i w1322 h1178 f0 v15 Ra-0.932450950145721 Rb0.500781953334808 Rc-0.763548374176025 Rd0.247554704546928 Re-0.10638951510191 Eev0 Er1 Eb1 r21.5824811614867 p-5.95754215157663 y10.5150658940951 TrX0 TrY0 TrZ0 Tpy0 Tpp0 j0 a0 b-0.00334547412904464 c0 d0 e0 g0 t0 Va1 Vb0.11472695878289 Vc-0.422861446956631 Vd0.320051426818116 Vx0 Vy0  Vm5 n"$FILE1"
#-hugin  cropFactor=1
i w1322 h1178 f0 v=0 Ra=0 Rb=0 Rc=0 Rd=0 Re=0 Eev0.0426115051251011 Er1.0319934794665 Eb1.00032481783156 r1.75586161826178 p-4.27676130487746 y-1.14582353775324 TrX0 TrY0 TrZ0 Tpy0 Tpp0 j0 a=0 b=0 c=0 d=0 e=0 g=0 t=0 Va=0 Vb=0 Vc=0 Vd=0 Vx=0 Vy=0  Vm5 n"$FILE2"
#-hugin  cropFactor=1
i w1322 h1178 f0 v=0 Ra=0 Rb=0 Rc=0 Rd=0 Re=0 Eev0.255316393679461 Er1.04208770233954 Eb1.03156271038965 r-18.2654938382702 p-2.75084004235145 y-14.1482953203335 TrX0 TrY0 TrZ0 Tpy0 Tpp0 j0 a=0 b=0 c=0 d=0 e=0 g=0 t=0 Va=0 Vb=0 Vc=0 Vd=0 Vx=0 Vy=0  Vm5 n"$FILE3"
#-hugin  cropFactor=1
i w1322 h1178 f0 v=0 Ra=0 Rb=0 Rc=0 Rd=0 Re=0 Eev1.63926994144708 Er1.0131035829039 Eb1.04491370266402 r-15.963869900671 p8.40256365161405 y-15.3227540740467 TrX0 TrY0 TrZ0 Tpy0 Tpp0 j0 a=0 b=0 c=0 d=0 e=0 g=0 t=0 Va=0 Vb=0 Vc=0 Vd=0 Vx=0 Vy=0  Vm5 n"$FILE4"
#-hugin  cropFactor=1
i w1322 h1178 f0 v=0 Ra=0 Rb=0 Rc=0 Rd=0 Re=0 Eev1.09546667568869 Er1.01662679260602 Eb1.03515372946557 r-0.26366251004473 p6.5341861446127 y-1.96860355184806 TrX0 TrY0 TrZ0 Tpy0 Tpp0 j0 a=0 b=0 c=0 d=0 e=0 g=0 t=0 Va=0 Vb=0 Vc=0 Vd=0 Vx=0 Vy=0  Vm5 n"$FILE5"
#-hugin  cropFactor=1
i w1322 h1178 f0 v=0 Ra=0 Rb=0 Rc=0 Rd=0 Re=0 Eev0.828535581105263 Er1.00605213421134 Eb1.02344610159655 r15.3748327739483 p5.41200541931162 y10.1876414748147 TrX0 TrY0 TrZ0 Tpy0 Tpp0 j0 a=0 b=0 c=0 d=0 e=0 g=0 t=0 Va=0 Vb=0 Vc=0 Vd=0 Vx=0 Vy=0  Vm5 n"$FILE6"


# specify variables that should be optimized
v Ra0
v Rb0
v Rc0
v Rd0
v Re0
v Vb0
v Vc0
v Vd0
v Eev1
v r1
v p1
v y1
v Eev2
v r2
v p2
v y2
v Eev3
v r3
v p3
v y3
v Eev4
v r4
v p4
v y4
v Eev5
v r5
v p5
v y5
v


# control points
c n0 N1 x22.0304602913646 y289.452138180294 X1195.67976440839 Y215.541868906387 t0
c n0 N1 x66.4046682473123 y473.832282499481 X1172.1666082052 Y399.50358312158 t0
c n0 N1 x24.8091375336804 y594.199915684824 X1092.73316870992 Y493.589552881278 t0
c n0 N1 x25.6726474129092 y790.201048672744 X1023.09187789167 Y671.202930417837 t0
c n0 N1 x25.0712443419824 y1008.3990235748 X945.093258378573 Y867.511231786806 t0
c n0 N1 x107.212553137292 y271.634389473598 X1280.25890881719 Y228.872009359618 t0
c n0 N1 x96.0052455170261 y466.741579943838 X1200.67927883301 Y403.382755810337 t0
c n0 N1 x89.6978337744452 y725.152338034742 X1104.51188601349 Y636.048865706786 t0
c n0 N1 x137.714292135607 y831.447678516607 X1111.1275614653 Y749.497038405101 t0
c n0 N1 x142.252951021429 y970.309071061313 X1065.77086462483 Y876.800916446099 t0
c n0 N1 x156.220576574337 y461.801536047328 X1259.5064719803 Y419.649140681423 t0
c n0 N1 x200.622076373892 y568.030459858834 X1262.95150174096 Y532.186902511753 t0
c n0 N1 x200.467690373536 y784.132317617062 X1185.74760995943 Y728.960059986497 t0
c n0 N1 x197.964838384298 y963.321572216583 X1119.40792733514 Y890.624059425234 t0
c n0 N1 x218.510020522514 y502.9419839498 X1302.81866687191 Y479.578474033602 t0
c n0 N1 x221.400110899163 y644.304688440662 X1254.3713680214 Y607.122856221213 t0
c n0 N1 x247.602592870424 y812.927446762233 X1219.07524051785 Y772.50814059447 t0
c n0 N1 x275.935138621089 y1003.74866990131 X1176.12940428199 Y955.342580349999 t0
c n0 N1 x289.982063104658 y719.976769961193 X1291.20494439383 Y703.113531167059 t0
c n0 N1 x279.824478145119 y741.962800794343 X1273.50556663158 Y718.515194616272 t0
c n0 N1 x280.417856242338 y962.674168236292 X1195.69087777579 Y920.543322893314 t0
c n1 N2 x55.1100274851752 y260.133081152792 X1250.21266477806 Y564.716663630875 t0
c n1 N2 x72.9125652992579 y442.143693811162 X1203.31033758249 Y736.285634825911 t0
c n1 N2 x47.3323374248067 y507.047174575982 X1157.39079704372 Y786.415311112722 t0
c n1 N2 x66.5834181673644 y665.193116486579 X1119.67825052955 Y934.989828695628 t0
c n1 N2 x67.7958989958164 y849.670723835365 X1055.5558738925 Y1103.22373561559 t0
c n1 N2 x96.0652336942808 y239.314228932444 X1295.87008403015 Y559.746744047482 t0
c n1 N2 x88.0584913433821 y424.889913648077 X1223.34406609967 Y726.033792784034 t0
c n1 N2 x137.183966748297 y453.918311280085 X1258.24447475209 Y769.494019257932 t0
c n1 N2 x96.9316530206328 y664.00527702761 X1148.69486762715 Y945.468818162087 t0
c n1 N2 x91.1535580597574 y777.805005714375 X1102.04916220568 Y1046.30276089752 t0
c n1 N2 x138.947062831461 y441.144262665986 X1264.6192193854 Y757.890535188293 t0
c n1 N2 x144.215445519849 y509.043827316232 X1244.72331048369 Y822.424300567295 t0
c n1 N2 x178.64368365593 y744.451740694548 X1195.05986712775 Y1047.68182806178 t0
c n1 N2 x178.684129824597 y744.71254361394 X1195.20671041119 Y1047.6167567889 t0
c n1 N2 x203.528187992808 y528.144577148518 X1292.85568958938 Y861.226535303916 t0
c n1 N2 x196.140825543351 y692.289342630041 X1229.57561786818 Y1007.02280293553 t0
c n1 N2 x205.822535821525 y770.546957159242 X1211.14194367091 Y1080.92670970099 t0
c n1 N2 x289.555446053771 y726.455841283999 X1304.35859483717 Y1071.60512721422 t0
c n1 N2 x307.880411116123 y764.183829201711 X1307.08724610129 Y1112.45803474294 t0
c n2 N3 x225.938979435856 y25.1416597261771 X35.0705621163634 Y1017.5965835548 t0
c n2 N3 x321.546591983337 y93.77828631577 X133.199523906972 Y1081.57216753202 t0
c n2 N3 x264.290373398345 y126.892564811485 X77.3638462630603 Y1117.30809876647 t0
c n2 N3 x252.024041870571 y141.544849707126 X65.6239892839634 Y1132.56264822983 t0
c n2 N3 x440.418199092261 y46.3377126742881 X249.804072158228 Y1029.08608011758 t0
c n2 N3 x566.979846150621 y83.0841361481524 X377.006479325084 Y1059.69666541476 t0
c n2 N3 x607.231282004145 y95.8666732023483 X418.207976930358 Y1070.01852855599 t0
c n2 N3 x473.903392580705 y167.287434250256 X288.254462386121 Y1147.48115235176 t0
c n2 N3 x533.302736830709 y175.691442421907 X348.265824724299 Y1153.14986709028 t0
c n2 N3 x663.063165212804 y33.9480303582395 X470.62257097864 Y1005.50589295586 t0
c n2 N3 x777.474449204986 y78.6636539669434 X586.365942566422 Y1043.79521270062 t0
c n2 N3 x864.277453550454 y105.062259564225 X673.666060041709 Y1066.69112317515 t0
c n2 N3 x661.566913931796 y160.40364523416 X475.27445573324 Y1131.45482915948 t0
c n2 N3 x831.098659567414 y193.753912082786 X646.498701797498 Y1155.69041096032 t0
c n2 N3 x898.352777403555 y44.7259120653542 X704.716417975914 Y1003.89311794119 t0
c n2 N3 x999.536754965382 y64.6888942454345 X806.866099057722 Y1018.27106782726 t0
c n2 N3 x904.571192691726 y113.940439778332 X715.093827416883 Y1072.33185729091 t0
c n2 N3 x1033.59771659257 y163.92183705336 X846.249697974904 Y1114.88259197724 t0
c n2 N3 x1022.97009439173 y184.163565786584 X836.753304899777 Y1135.44277972171 t0
c n2 N3 x1231.98533960169 y44.9446989271489 X1035.83219404586 Y985.329762488883 t0
c n2 N3 x1192.86695651363 y68.4731847166141 X998.475942582574 Y1010.99311360577 t0
c n2 N3 x1203.89076648244 y132.674481945786 X1013.33053053828 Y1073.90637584462 t0
c n2 N3 x1133.4737100903 y162.642944648532 X945.087151244171 Y1107.75946829581 t0
c n2 N3 x1141.33919892817 y201.559672578704 X955.549059840533 Y1145.96815835673 t0
c n3 N4 x1119.20776688417 y957.546366500165 X32.6236381315532 Y650.258487899985 t0
c n3 N4 x1076.28393886867 y1138.11431265373 X35.9956424338044 Y840.535613269307 t0
c n3 N4 x1144.25937204406 y846.868857665827 X29.6863656246311 Y534.229633990277 t0
c n3 N4 x1149.9088814047 y864.158645330651 X39.6929251471048 Y550.019821865726 t0
c n3 N4 x1150.57901114 y1027.95834631725 X80.9160532640468 Y712.093855662216 t0
c n3 N4 x1205.28615417192 y658.389957189824 X43.3052125672141 Y331.726877318897 t0
c n3 N4 x1172.57211821953 y823.106971363812 X51.5284625140321 Y503.532645529234 t0
c n3 N4 x1183.44830764365 y858.448910841211 X70.2724092701514 Y535.288813947804 t0
c n3 N4 x1211.08091577696 y1098.33147724489 X158.331537864481 Y766.424613119811 t0
c n3 N4 x1260.35994512937 y457.767770995966 X48.2411540454679 Y117.353784214162 t0
c n3 N4 x1257.28575633243 y574.618321247505 X74.2390681061743 Y235.43498862106 t0
c n3 N4 x1243.07140405614 y716.625255871267 X94.7585969498896 Y380.325779759905 t0
c n3 N4 x1221.58200708153 y957.56402463303 X132.985764917851 Y624.65760274429 t0
c n3 N4 x1244.2513942123 y1016.70653297378 X169.502943557724 Y677.742356179951 t0
c n3 N4 x1289.34395948666 y422.56294217962 X69.2010144873732 Y75.0864497797055 t0
c n3 N4 x1264.15564713177 y648.889768463471 X99.371008989015 Y307.358602540623 t0
c n3 N4 x1291.50822793378 y701.21289669181 X138.802222747543 Y353.329659270955 t0
c n3 N4 x1295.03671250541 y898.284322159713 X190.553257348341 Y547.651992493869 t0
c n3 N4 x1280.17263223592 y1125.55523702371 X232.270430763123 Y775.111529149476 t0
c n4 N5 x1019.36313991077 y895.890563423739 X31.0206855342173 Y976.95645986092 t0
c n4 N5 x981.387599334832 y1044.1147092786 X32.5359526574971 Y1134.61820855164 t0
c n4 N5 x1104.57000387117 y684.19807417136 X60.1231155423875 Y744.85663094763 t0
c n4 N5 x1066.63267357729 y839.840855046117 X62.9251738165308 Y909.197181390443 t0
c n4 N5 x1102.31294927018 y971.117203688558 X131.986151189529 Y1029.79466941449 t0
c n4 N5 x1161.61186482011 y417.264464384907 X49.2577479847326 Y465.315134439957 t0
c n4 N5 x1145.27795247363 y642.297690459291 X89.6590851631687 Y691.311193314058 t0
c n4 N5 x1172.53050007686 y916.874532589241 X186.489213646209 Y957.723195856205 t0
c n4 N5 x1153.93469333539 y937.556246717814 X174.609938797443 Y983.495060298682 t0
c n4 N5 x1214.72044066816 y113.858408090724 X26.5774092747987 Y152.82276255096 t0
c n4 N5 x1191.44913785517 y340.338092371982 X59.1434761191062 Y382.311115560543 t0
c n4 N5 x1204.44448079935 y622.110043932532 X142.682231763688 Y658.291076212134 t0
c n4 N5 x1184.74715129803 y906.734865584191 X195.803290757428 Y945.094811745676 t0
c n4 N5 x1182.48518450705 y1069.58562145266 X235.601913969983 Y1105.57525930314 t0
c n4 N5 x1268.0003015727 y42.6960472279949 X62.4598070914086 Y68.5040712779421 t0
c n4 N5 x1277.97954810034 y332.554514605515 X141.882433341138 Y353.451379069489 t0
c n4 N5 x1293.00514256075 y627.194998780567 X230.09308734192 Y640.257325325212 t0
c n4 N5 x1296.22059924325 y910.917173335927 X305.091303968964 Y919.651355494683 t0
c n4 N5 x1273.76288288228 y1018.89644416701 X311.252739881512 Y1031.5265041012 t0
c n0 N5 x141.515422541674 y43.1410905816147 X499.042992021046 Y954.517682890095 t0
c n0 N5 x199.180683898465 y92.2925305582774 X551.588108885567 Y1008.04258934712 t0
c n0 N5 x195.758130392699 y157.805919735403 X541.384457103794 Y1071.59090101687 t0
c n0 N5 x126.268666930666 y164.985115510256 X471.861547888173 Y1071.4721445351 t0
c n0 N5 x122.92751758743 y211.935602390047 X463.824501997229 Y1117.4615936849 t0
c n0 N5 x229.683588574969 y38.3069856264196 X585.81916993541 Y957.797214971405 t0
c n0 N5 x213.94988385851 y89.183695865082 X565.358390162936 Y1006.70648148231 t0
c n0 N5 x221.078158945107 y123.565803928069 X568.84395606838 Y1040.11989574821 t0
c n0 N5 x232.378889719305 y184.539993622126 X574.11338456682 Y1101.19242220131 t0
c n0 N5 x224.091885213011 y217.499239389493 X562.160791046955 Y1132.35692010433 t0
c n0 N5 x479.75582713052 y55.3652338127053 X829.611266560643 Y998.772985180831 t0
c n0 N5 x551.513604704009 y111.512188647212 X895.637703782715 Y1059.61853419608 t0
c n0 N5 x544.656796542633 y156.987026801267 X884.781530360904 Y1103.96144987899 t0
c n0 N5 x514.131788448002 y174.75306146351 X853.049812013856 Y1118.12308281132 t0
c n0 N5 x446.150102079919 y227.046127283131 X780.96106146574 Y1163.74739295735 t0
c n0 N5 x600.699278620481 y35.0370813257085 X951.239575851546 Y989.074424828574 t0
c n0 N5 x635.546181829399 y109.811469613013 X978.557741342445 Y1065.42018884126 t0
c n0 N5 x634.760755215426 y114.566843776649 X977.714240540189 Y1071.13553172928 t0
c n0 N5 x608.485902384827 y190.43198432848 X945.309436264986 Y1142.30465680561 t0
c n0 N5 x787.13960146489 y50.1606589140911 X1133.8135399714 Y1020.7027091799 t0
c n0 N5 x790.64884407214 y76.3906939783566 X1134.844514253 Y1046.54600270672 t0
c n0 N5 x911.507329900459 y161.394003828781 X1247.77495846746 Y1140.8086534883 t0
c n0 N5 x922.470467065578 y162.095790670595 X1258.93996698559 Y1142.20773770846 t0
c n1 N4 x123.24208970803 y13.6679709463864 X216.092443927822 Y955.975521602665 t0
c n1 N4 x257.03202826728 y70.4994518927941 X347.031158022583 Y1014.98452645022 t0
c n1 N4 x157.455909581547 y101.685344080105 X247.196416519824 Y1043.20750004214 t0
c n1 N4 x233.782325567442 y149.008567349081 X321.003200644868 Y1092.14671601297 t0
c n1 N4 x176.018972355367 y209.514006409857 X262.831758468891 Y1151.572537329 t0
c n1 N4 x434.649233027979 y47.00282444792 X523.799829915 Y996.160978376552 t0
c n1 N4 x328.208334780634 y63.9348793962856 X417.753103163265 Y1010.45647549845 t0
c n1 N4 x380.58293799011 y102.48227859711 X468.831111919087 Y1050.08359698039 t0
c n1 N4 x316.644869790172 y135.06545767501 X404.167273662403 Y1080.80533407888 t0
c n1 N4 x414.706646671215 y199.254151185655 X500.106009066894 Y1146.20358416001 t0
c n1 N4 x544.728654277667 y29.1060295937708 X633.309170398658 Y980.874182392822 t0
c n1 N4 x734.653988665738 y62.8739316536819 X820.683277104835 Y1017.1832439593 t0
c n1 N4 x679.510905236524 y117.235077782283 X764.937646857126 Y1070.44027935413 t0
c n1 N4 x660.240324078788 y140.512777464033 X745.493390143453 Y1093.11327679389 t0
c n1 N4 x738.988364261486 y197.427521964165 X822.978985382503 Y1150.82010625085 t0
c n1 N4 x978.462855456117 y49.0607244499837 X1063.1934938856 Y1007.70869128398 t0
c n1 N4 x911.539374024786 y91.5238534552293 X996.527032187605 Y1048.43488333684 t0
c n1 N4 x953.55865636887 y113.310751799623 X1037.49562567647 Y1071.0169645194 t0
c n1 N4 x801.915913236541 y162.707770314979 X886.335604437771 Y1117.45271859565 t0
c n1 N4 x851.139347591551 y195.60105884315 X934.901275714664 Y1150.44317866371 t0
c n1 N4 x1000.2009671067 y45.583657789128 X1084.53774178885 Y1004.42515633889 t0
c n1 N4 x1190.61327099355 y58.2412013412503 X1273.76288288228 Y1018.89644416701 t0
c n1 N4 x1062.35231168295 y124.187027157613 X1145.56477805677 Y1083.03173608884 t0
c n1 N4 x1045.4338688994 y141.550501428662 X1128.94288650014 Y1100.03997015608 t0
c n1 N4 x1029.86067856077 y180.206313541288 X1113.03563596655 Y1138.22012132843 t0
c n1 N5 x911.539374024786 y91.5238534552293 X48.3351181108685 Y1134.53072247474 t0
c n1 N5 x953.55865636887 y113.310751799623 X94.5164832740747 Y1145.32268461171 t0
c n1 N5 x1012.18544225149 y41.1434979302702 X133.203059459878 Y1060.3299013506 t0
c n1 N5 x980.041364211967 y100.709043729968 X117.408298580309 Y1126.31278953087 t0
c n1 N5 x1009.70806712675 y112.276936339125 X148.618400851794 Y1129.73504961956 t0
c n1 N5 x1081.59786887326 y60.1048233713765 X206.905255102493 Y1061.72927224767 t0
c n1 N5 x1074.56329716893 y71.0847550629369 X202.088767311449 Y1073.56063239014 t0
c n1 N5 x1062.35231168295 y124.187027157613 X203.06145943261 Y1128.63020835476 t0
c n1 N5 x1131.09586185946 y142.794300685907 X274.920421463352 Y1128.79439680152 t0
c n1 N5 x1190.61327099355 y58.2412013412503 X311.252739881512 Y1031.5265041012 t0
c n1 N5 x1141.88617067697 y93.6311943004548 X272.949184191445 Y1078.45163840984 t0
c n1 N5 x1162.36744288155 y137.012536480615 X302.720315544522 Y1114.08257807378 t0
c n1 N5 x1192.99389100951 y171.805202667962 X342.184148581698 Y1141.35141458621 t0
c n1 N5 x1207.9335063534 y186.775559219498 X360.327753635898 Y1151.58428735785 t0
c n1 N5 x1218.47657996163 y63.5605359914566 X340.208481395809 Y1029.14074101924 t0
c n1 N5 x1228.34648383926 y66.107260150088 X349.936743428979 Y1029.61903742611 t0
c n1 N5 x1287.18259487528 y133.751792731273 X423.922799158378 Y1080.27303744179 t0
c n1 N5 x1287.62043632363 y173.203174801578 X434.433471537179 Y1118.41131901891 t0
c n1 N5 x1259.9048442495 y201.612869213445 X414.793669632548 Y1153.53861361405 t0
c n2 N4 x1202.1155938831 y326.523917239826 X17.7139053571639 Y981.511362779567 t0
c n2 N4 x1213.48537692728 y348.190414246896 X35.1769975480061 Y998.479697628876 t0
c n2 N4 x1234.05661284355 y318.42377765104 X45.8004341296382 Y962.848755971685 t0
c n2 N4 x1242.70300176138 y248.861371990374 X32.2500336534297 Y892.957018973175 t0
c n2 N4 x1254.21262398913 y343.068826785634 X72.6807073072576 Y980.828440287804 t0
c n2 N4 x1239.04170232839 y349.750799080011 X60.357678328054 Y991.933413402123 t0
c n2 N4 x1270.0309916131 y166.166438258679 X31.8258376349847 Y804.197730314459 t0
c n2 N4 x1263.13184964509 y200.687407442188 X35.9956424338044 Y840.535613269307 t0
c n2 N4 x1259.81494246386 y328.954195797487 X73.6635949457545 Y964.887169573191 t0
c n2 N4 x1261.84813905212 y357.187531253878 X84.1368601452879 Y992.243816535373 t0
c n2 N4 x1258.53160505619 y454.667885872427 X112.689772831038 Y1087.42938541302 t0
c n2 N4 x1279.81438880127 y109.675013886279 X23.4520853066275 Y746.471304351075 t0
c n2 N4 x1287.47924739467 y206.750943229202 X61.449815590085 Y837.86771117686 t0
c n2 N4 x1282.75783566715 y281.065576292776 X80.0195323346112 Y911.525521889531 t0
c n2 N4 x1282.74918252431 y360.327843031168 X105.789426250279 Y988.519076903379 t0

#hugin_optimizeReferenceImage 0
#hugin_blender enblend
#hugin_remapper nona
#hugin_enblendOptions
#hugin_enfuseOptions
#hugin_hdrmergeOptions -m avg -c
#hugin_verdandiOptions
#hugin_outputLDRBlended true
#hugin_outputLDRLayers false
#hugin_outputLDRExposureRemapped false
#hugin_outputLDRExposureLayers false
#hugin_outputLDRExposureBlended false
#hugin_outputLDRStacks false
#hugin_outputLDRExposureLayersFused false
#hugin_outputHDRBlended false
#hugin_outputHDRLayers false
#hugin_outputHDRStacks false
#hugin_outputLayersCompression LZW
#hugin_outputImageType tif
#hugin_outputImageTypeCompression LZW
#hugin_outputJPEGQuality 90
#hugin_outputImageTypeHDR exr
#hugin_outputImageTypeHDRCompression LZW
#hugin_outputStacksMinOverlap 0.7
#hugin_outputLayersExposureDiff 0.5
#hugin_outputRangeCompression 0
#hugin_optimizerMasterSwitch 1
#hugin_optimizerPhotoMasterSwitch 21

EOF

# Note: Assistant will produce undesirable output by modifying the projection and cropping.
#hugin_executor --assistant --prefix $PREFIX ${PREFIX}.pto

hugin_executor --stitching --prefix $PREFIX ${PREFIX}.pto
