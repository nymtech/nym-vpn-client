import Foundation

public enum OnDemandSSID {
    case any
    case onlySpecific([String])
    case onWifiAndCellular([String])
    //    case onWifiAndEthernet
}
