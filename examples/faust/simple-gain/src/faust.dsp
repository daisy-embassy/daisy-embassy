declare name "LPVol";

import("stdfaust.lib");
fq = hslider("CutOff", 70, 0, 127, 1) : ba.midikey2hz; // removed control smoothing and visualizaion
q = hslider("Resonance", 1, 0, 3, 0.001) : si.smoo;
invol(i) = _ : _;  // pass through, removed VU meter
outvol(i) = _ : _; // pass through, removed VU meter

process = _,_  : par(i, 2, hgroup("ClackFaust",
									hgroup("0 in vol",invol(i)):
									hgroup("1 controls",
										vgroup("filter",
                                        fi.svf.lp(fq, q))
										*
										(vslider("gain",0,-70,10,0.001):ba.db2linear: si.smoo))
										<:
									hgroup("2 out vol",outvol(i))));