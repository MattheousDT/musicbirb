import React, { PropsWithChildren, useMemo, useRef, useState } from "react";
import {
	Dimensions,
	FlatList,
	ListRenderItem,
	StyleSheet,
	View,
} from "react-native";

type Props<T extends Record<any, any>> = {
	data: T[];
	perPage: number;
	renderItem: ListRenderItem<T> | null | undefined;
	keyExtractor?: ((item: any[], index: number) => string) | undefined;
};

const chunkArray = (arr: any[], size: number) => {
	const chunks = [];
	if (!arr) return [];
	for (let i = 0; i < arr.length; i += size) {
		chunks.push(arr.slice(i, i + size));
	}
	return chunks;
};

const { width } = Dimensions.get("window");

export function PaginatedList<T extends Record<any, any>>({
	data,
	perPage,
	renderItem,
	...props
}: Props<T> & Partial<Omit<PropsWithChildren<FlatList>, keyof Props<any>>>) {
	const [currentIndex, setCurrentIndex] = useState(0);

	const onViewableItemsChanged = useRef(({ viewableItems }: any) => {
		if (viewableItems.length > 0) {
			setCurrentIndex(viewableItems[0].index || 0);
		}
	}).current;

	const viewabilityConfig = useRef({
		viewAreaCoveragePercentThreshold: 50,
	}).current;

	const chunked = useMemo(() => chunkArray(data, perPage), [data, perPage]);

	return (
		<View style={styles.container}>
			<FlatList
				{...props}
				horizontal
				pagingEnabled
				showsHorizontalScrollIndicator={false}
				onViewableItemsChanged={onViewableItemsChanged}
				viewabilityConfig={viewabilityConfig}
				data={chunked}
				renderItem={({ item: page, separators }) => (
					<View style={styles.listContainer}>
						{page.map((item, index) =>
							renderItem?.({ item, index, separators }),
						)}
					</View>
				)}
			/>
			{chunked.length > 1 && (
				<View style={styles.paginationContainer}>
					{chunked.map((_, index) => {
						const isActive = currentIndex === index;
						return (
							<View
								key={index}
								style={[
									styles.dot,
									isActive ? styles.dotActive : styles.dotInactive,
								]}
							/>
						);
					})}
				</View>
			)}
		</View>
	);
}

const styles = StyleSheet.create({
	container: {
		display: "flex",
		flexDirection: "column",
		gap: 16,
	},
	listContainer: { width: width, paddingInline: 24, gap: 4 },
	paginationContainer: {
		flexDirection: "row",
		justifyContent: "center",
		alignItems: "center",
	},
	dot: {
		width: 8,
		height: 8,
		borderRadius: 4,
		marginHorizontal: 4,
	},
	dotActive: {
		backgroundColor: "#1e293b",
		borderWidth: 0,
	},
	dotInactive: {
		backgroundColor: "transparent",
		borderWidth: 1,
		borderColor: "#1e293b",
	},
});
