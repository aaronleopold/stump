interface ReadProgress {
	/**
	 * The id of the media file this progress belongs to.
	 */
	mediaId: string;
	/**
	 * The id of the user this progress belongs to.
	 */
	userId: string;
	/**
	 * The current page the user is on.
	 */
	page: number;
}
