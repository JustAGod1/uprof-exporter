![Docker Pulls](https://img.shields.io/docker/pulls/justagod/amd-uprof-exporter)

# AMD uProf Prometheus exporter

Runs `AMDuProfPcm` every few seconds and exports counters in Prometheus format. 
In Gauge format to be precise.


Requires `msr` to be loaded: `modprobe msr`