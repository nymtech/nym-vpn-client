package net.nymtech.qrscanner.model

// Shown under the viewfinder and on the permission and camera error screens.
data class QrScanAction(val label: String, val onClick: () -> Unit)
