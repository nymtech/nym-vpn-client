package net.nymtech.nymvpn.ui.screens.scan

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.nymtech.qrscanner.QrScanValidator
import javax.inject.Inject

@HiltViewModel
class QrScanViewModel @Inject constructor(validator: ScannedCodeValidator) :
	ViewModel(),
	QrScanValidator by validator
