import SwiftUI
import Theme

struct CityPopoverView: View {
    let cluster: CityCluster
    let onSelect: () -> Void

    var body: some View {
        VStack(spacing: 8) {
            Text(cluster.city)
                .font(.subheadline.bold())
                .foregroundStyle(NymColor.primary)

            Text("\(cluster.nodeCount) \(cluster.nodeCount == 1 ? "node" : "nodes")")
                .font(.caption)
                .foregroundStyle(NymColor.gray1)

            Button(action: onSelect) {
                Text("Select")
                    .font(.caption.bold())
                    .foregroundStyle(NymColor.black)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 6)
                    .background(NymColor.accent, in: RoundedRectangle(cornerRadius: 8))
            }
            .buttonStyle(.plain)
        }
        .padding(12)
        .background(NymColor.elevation, in: RoundedRectangle(cornerRadius: 12))
    }
}
