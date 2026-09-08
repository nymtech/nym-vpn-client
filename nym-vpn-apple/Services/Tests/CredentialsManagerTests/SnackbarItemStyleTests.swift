import Testing
import SnackbarManager

struct SnackbarItemStyleTests {
    @Test func criticalStyleShowsCloseButton() {
        #expect(SnackbarItem.Style.critical.showsCloseButton)
    }

    @Test func nonCriticalStylesShowCloseButton() {
        #expect(SnackbarItem.Style.confirmation.showsCloseButton)
        #expect(SnackbarItem.Style.warning.showsCloseButton)
        #expect(SnackbarItem.Style.neutral.showsCloseButton)
        #expect(SnackbarItem.Style.negative.showsCloseButton)
    }
}
