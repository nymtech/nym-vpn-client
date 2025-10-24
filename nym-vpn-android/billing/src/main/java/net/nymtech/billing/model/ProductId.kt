package net.nymtech.billing.model

enum class ProductId(val value: String) {
	Monthly("nym.monthly"),
	Yearly("nym.yearly"),
	;

	companion object {
		fun fromId(id: String): ProductId? = values().firstOrNull { it.value == id }
	}
}
