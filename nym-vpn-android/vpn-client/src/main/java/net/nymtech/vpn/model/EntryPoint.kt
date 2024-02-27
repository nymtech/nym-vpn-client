package net.nymtech.vpn.model

sealed class EntryPoint {
    //TODO enforce only two char countryISO
    data class Location(private val location: String) : EntryPoint() {

        //TODO make this serialize later
        override fun toString(): String {
            return "{ \"Location\": { \"location\": \"${location}\" }}"
        }
    }
    //TODO impl later
    private sealed class Gateway() : EntryPoint()
}