# Enhanced Entenna Status

## Description

UI util that checks modem status and shows history bar plot of signal strength parameters. This data may be helpful during precise antenna pointing or positioning the modem when using internal antenna only.

![Dashboard with Netgear modem](images/ac785s.png)

Checking history of download/upload bandwidth and signal params:

![Checking dl/ul bandwidth](images/ac785s-2.png)

![Checking signal status](images/ac785s-3.png)

Information about Huawei modem:

![Checking status of Huawei modem](images/e5573.png)

Supported manufacturers:
* Netgear
* Huawei

Tested on:
* Netgear: MR2100, AC785S
* Huawei: E8372h-608, E5573s-320

## Video demo

[![Demo of Enhanced Antenna Status utility](https://img.youtube.com/vi/M9-LlXhgATA/maxresdefault.jpg)](https://youtu.be/M9-LlXhgATA)

## TODO

- UI enhancements
  - [x] Hide unused info (e.g. 3G in LTE mode)
  - [x] Adjust poll timeout (longer timeouts to reduce load on the modem)
- [ ] Create pre-filled list of host addresses: scan networks and get gateways
- [ ] Additional info for Netgear from Telnet (channels and band widths) (need to resolve long timeout issues)
- [x] Download/upload bar plot

## Links

* JS plugins for Web dashboards: https://github.com/Postrediori/HuaweiMobileDashboard
