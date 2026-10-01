package net.nymtech.billing.model

enum class ProductId(val value: String) {
	Monthly("nym.monthly"),
	SixMonths("nym.six_months"),
	Yearly("nym.yearly"),
	;

	companion object {
		fun fromId(id: String): ProductId? = values().firstOrNull { it.value == id }
	}
}
