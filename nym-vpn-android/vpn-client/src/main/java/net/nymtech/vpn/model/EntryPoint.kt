package net.nymtech.vpn.model

sealed class EntryPoint {
    //TODO enforce only two char countryISO
    data class Location(private val location: String) : EntryPoint() {

        //TODO make this serialize later
        override fun toString(): String {
            return "{ \"Location\": { \"location\": \"${location}\" }}"
        }
    }
    private sealed class Gateway() : EntryPoint() {

        //TODO example, impl later
        override fun toString(): String {
            return "{ \"Gateway\": { \"identity\": [94,69,76,90,128,87,76,174,15,177,79,44,11,234,27,225,205,162,191,216,144,29,74,210,50,62,121,13,154,85,209,40] }}"
        }
    }
}