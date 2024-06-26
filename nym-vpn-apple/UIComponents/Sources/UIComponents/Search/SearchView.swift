import SwiftUI
import Theme

public struct SearchView: View {
    private let strokeTitle = "search".localizedString
    private let searchCountryTitle = "searchCountry".localizedString
    private let searchImageName = "searchIcon"

    @FocusState.Binding private var isSearchFocused: Bool

    @Binding var searchText: String

    public init(searchText: Binding<String>, isSearchFocused: FocusState<Bool>.Binding) {
        _searchText = searchText
        _isSearchFocused = isSearchFocused
    }

    public var body: some View {
        StrokeBorderView(strokeTitle: strokeTitle) {
            HStack {
                searchImage()
                searchTextfield()
                Spacer()
            }
        }
        .onTapGesture {
            isSearchFocused = true
        }
#if os(iOS)
        .defersSystemGestures(on: .`vertical`)
#endif
    }
}

extension SearchView {
    @ViewBuilder
    func searchImage() -> some View {
        Image(searchImageName, bundle: .module)
            .resizable()
            .frame(width: 24, height: 24)
            .cornerRadius(50)
            .padding(EdgeInsets(top: 0, leading: 12, bottom: 0, trailing: 0))
    }

    @ViewBuilder
    func searchTextfield() -> some View {
        ZStack(alignment: .leading) {
            if searchText.isEmpty {
                Text(searchCountryTitle)
                    .foregroundStyle(NymColor.sysOutline)
                    .textStyle(.Body.Large.regular)
            }
            TextField("", text: $searchText)
                .foregroundStyle(NymColor.sysOnSurface)
                .textFieldStyle(PlainTextFieldStyle())
                .textStyle(.Body.Large.primary)
                .focused($isSearchFocused)
        }
    }
}
