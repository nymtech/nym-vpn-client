import SwiftUI
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

public extension Color {
    /// Nym 2.0 semantic color tokens — mirrors the Figma `Colors` collection
    /// (Dark & Light modes). Source of truth: `nym-color-tokens` spec, 2026-05-29.
    enum Nym {
        // MARK: - Brand · Primary
        /// `primary` — Spring Green CTA. D #5BF0A0 / L #1ED674
        public static let primary         = Color("Nym.Primary", bundle: .module)
        /// `primary-hover` — pointer-over. D #4AD88C / L #158551
        public static let primaryHover    = Color("Nym.PrimaryHover", bundle: .module)
        /// `primary-pressed` — pressed. D #2DE783 / L #0F6E42
        public static let primaryPressed  = Color("Nym.PrimaryPressed", bundle: .module)
        /// `primary-active` — active/selected. #1ED674 universal
        public static let primaryActive   = Color("Nym.PrimaryActive", bundle: .module)
        /// `primary-disabled` — muted CTA. #B4DAC6 universal
        public static let primaryDisabled = Color("Nym.PrimaryDisabled", bundle: .module)
        /// `primary-text` — fg/icon on primary fill. #0B0B0B universal
        public static let primaryText     = Color("Nym.PrimaryText", bundle: .module)

        // MARK: - Surface & Structure
        /// `background` — page. D #090909 / L #FFFFFF
        public static let background      = Color("Nym.Background", bundle: .module)
        /// `surface` — card / panel / modal. D #1D1D1F / L #F5F5FA
        public static let surface         = Color("Nym.Surface", bundle: .module)
        /// `surface-alt` — raised / hover surface. D #262628 / L #EEEEF6
        public static let surfaceAlt      = Color("Nym.SurfaceAlt", bundle: .module)
        /// `surface-pressed` — pressed / sunken surface. D #141414 / L #E8E8F2
        public static let surfacePressed  = Color("Nym.SurfacePressed", bundle: .module)
        /// `border` — outline / stroke. D #3A3A3C / L #DCDCE6
        public static let border          = Color("Nym.Border", bundle: .module)
        /// `divider` — 1px separators. D #3A3A3C / L #EBEBF2
        public static let divider         = Color("Nym.Divider", bundle: .module)

        // MARK: - Typography
        /// `text-primary` — body + headings. D #FFFFFF / L #0A0A0A
        public static let textPrimary     = Color("Nym.TextPrimary", bundle: .module)
        /// `text-secondary` — labels, meta. D #AEACB1 / L #5A5A60
        public static let textSecondary   = Color("Nym.TextSecondary", bundle: .module)
        /// `text-tertiary` — state, hint. D #8B8B90 / L #70707A
        public static let textTertiary    = Color("Nym.TextTertiary", bundle: .module)
        /// `text-disabled` — disabled copy. D #6C6C6F / L #A8A8B0
        public static let textDisabled    = Color("Nym.TextDisabled", bundle: .module)

        // MARK: - Status · Success
        /// `success` — connected, ok. #28C96C universal
        public static let success         = Color("Nym.Success", bundle: .module)
        /// `success-hover` — #22B460 universal
        public static let successHover    = Color("Nym.SuccessHover", bundle: .module)
        /// `success-pressed` — #1C9E55 universal
        public static let successPressed  = Color("Nym.SuccessPressed", bundle: .module)
        /// `success-disabled` — #AEEBC8 universal
        public static let successDisabled = Color("Nym.SuccessDisabled", bundle: .module)

        // MARK: - Status · Warning
        /// `warning` — caution. #FFCC33 universal
        public static let warning         = Color("Nym.Warning", bundle: .module)
        /// `warning-hover` — #E6B82E universal
        public static let warningHover    = Color("Nym.WarningHover", bundle: .module)
        /// `warning-pressed` — #CC9F27 universal
        public static let warningPressed  = Color("Nym.WarningPressed", bundle: .module)
        /// `warning-disabled` — #FFF2BF universal
        public static let warningDisabled = Color("Nym.WarningDisabled", bundle: .module)

        // MARK: - Status · Error
        /// `error` — failure. #E73E14 universal
        public static let error           = Color("Nym.Error", bundle: .module)
        /// `error-hover` — #D33712 universal
        public static let errorHover      = Color("Nym.ErrorHover", bundle: .module)
        /// `error-pressed` — #C03110 universal
        public static let errorPressed    = Color("Nym.ErrorPressed", bundle: .module)
        /// `error-disabled` — #F0B8AB universal
        public static let errorDisabled   = Color("Nym.ErrorDisabled", bundle: .module)

        // MARK: - Status · Info
        /// `info` — info, neutral accent. #485ECA universal
        public static let info            = Color("Nym.Info", bundle: .module)
        /// `info-hover` — #3F53B7 universal
        public static let infoHover       = Color("Nym.InfoHover", bundle: .module)
        /// `info-pressed` — #3748A3 universal
        public static let infoPressed     = Color("Nym.InfoPressed", bundle: .module)
        /// `info-disabled` — #C5CDF8 universal
        public static let infoDisabled    = Color("Nym.InfoDisabled", bundle: .module)

        // MARK: - Link
        /// `link` — hyperlink. #2D7BFF universal
        public static let link            = Color("Nym.Link", bundle: .module)
        /// `link-hover` — #2366D6 universal
        public static let linkHover       = Color("Nym.LinkHover", bundle: .module)
        /// `link-pressed` — #1A50AA universal
        public static let linkPressed     = Color("Nym.LinkPressed", bundle: .module)
        /// `link-visited` — #6A66FF universal
        public static let linkVisited     = Color("Nym.LinkVisited", bundle: .module)
    }
}
