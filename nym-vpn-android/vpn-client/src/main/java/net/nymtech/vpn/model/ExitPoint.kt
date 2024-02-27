package net.nymtech.vpn.model

sealed class ExitPoint {
    data class Location(private val location: String) : ExitPoint() {
        override fun toString(): String {
            return "{ \"Location\": { \"location\": \"${location}\" }}"
        }
    }
    //TODO impl later
    private sealed class Gateway() : ExitPoint()
    private sealed class Address() : ExitPoint()

}