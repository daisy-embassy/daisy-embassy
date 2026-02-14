declare name "LPVol";

import("stdfaust.lib");

process = _,_  : par(i, 2, hgroup("1 controls", _ * (vslider("gain",0,-30,10,0.001):ba.db2linear: si.smoo)));

