import SwiftUI

struct PaginatedList<Item, Content: View>: View {
	let items: [Item]
	let itemsPerPage: Int
	let rowHeight: CGFloat
	let content: (Item) -> Content

	@Environment(\.horizontalSizeClass) var horizontalSizeClass

	var columns: Int {
		horizontalSizeClass == .regular ? 2 : 1
	}

	var effectiveItemsPerPage: Int {
		itemsPerPage * columns
	}

	var chunks: [[Item]] {
		stride(from: 0, to: items.count, by: effectiveItemsPerPage).map {
			Array(items[$0..<min($0 + effectiveItemsPerPage, items.count)])
		}
	}

	// Dynamically calculate the maximum rows needed so we don't have massive whitespace
	var actualRowsNeeded: Int {
		min(items.count, itemsPerPage)
	}

	var body: some View {
		if items.isEmpty {
			EmptyView()
		} else {
			TabView {
				ForEach(0..<chunks.count, id: \.self) { chunkIndex in
					VStack(spacing: 0) {
						// The White Container (Grouped List Style)
						HStack(alignment: .top, spacing: 16) {
							ForEach(0..<columns, id: \.self) { colIndex in
								let startIndex = colIndex * itemsPerPage
								let endIndex = min(startIndex + itemsPerPage, chunks[chunkIndex].count)

								if startIndex < chunks[chunkIndex].count {
									VStack(spacing: 0) {
										ForEach(startIndex..<endIndex, id: \.self) { itemIndex in
											content(chunks[chunkIndex][itemIndex])
												.frame(height: rowHeight)
												.frame(maxWidth: .infinity, alignment: .leading)

											if itemIndex < endIndex - 1 {
												Divider()
											}
										}
									}
									.background(Color(UIColor.secondarySystemGroupedBackground))
									.clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
								} else {
									Spacer().frame(maxWidth: .infinity)
								}
							}
						}
						.padding(.horizontal, 16)  // Aligns with the "Home" padding exactly

						Spacer(minLength: 0)  // Pushes container to the top, reserving the bottom area for the dots
					}
				}
			}
			.tabViewStyle(.page(indexDisplayMode: .automatic))
			// Height = (number of visible rows * height) + 40pts reserved for the dots outside the box
			.frame(height: CGFloat(actualRowsNeeded) * rowHeight + 40)
			.onAppear {
				UIPageControl.appearance().currentPageIndicatorTintColor = UIColor.label
				UIPageControl.appearance().pageIndicatorTintColor = UIColor.systemGray4
			}
		}
	}
}

struct RowButtonStyle: ButtonStyle {
	func makeBody(configuration: Configuration) -> some View {
		configuration.label
			.background(configuration.isPressed ? Color(UIColor.systemGray5) : Color.clear)
			.contentShape(Rectangle())
	}
}
