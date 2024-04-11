import Foundation

public enum OnDemandRule {
    case off
    case onWifi(OnDemandSSID)
    case onWifiAndCellular(OnDemandSSID)
    // case onWifiAndEthernet
}
