import React, { useMemo } from 'react';
import { useQuery } from 'react-query';
import { Navigate, Outlet, useLocation } from 'react-router-dom';
import { getLibraries } from '~api/query/library';
import Lazy from '~components/Lazy';
import Topbar from '~components/Topbar';
import Sidebar from '~components/Sidebar/Sidebar';
import { useStore } from '~store/store';

import { Box, Flex, useColorModeValue } from '@chakra-ui/react';
import { AxiosError } from 'axios';
import client from '~api/client';
import { useUser } from '~hooks/useUser';

export default function MainLayout() {
	const location = useLocation();

	const setLibraries = useStore((state) => state.setLibraries);

	const _user = useUser();

	const { isLoading, error } = useQuery('getLibraries', getLibraries, {
		onSuccess(res) {
			setLibraries(res.data.data);
		},
		onError(err: AxiosError) {
			// 401 errors will be handled below
			if (err.response?.status !== 401) {
				throw new Error(err.message);
			}
		},
		// Send all non-401 errors to the error page
		useErrorBoundary: (err: AxiosError) => !err || (err.response?.status ?? 500) !== 401,
	});

	const hideSidebar = useMemo(() => {
		// hide sidebar when on /books/:id/pages/:page or /epub/
		// TODO: replace with single regex, I am lazy rn
		return (
			location.pathname.match(/\/books\/.+\/pages\/.+/) || location.pathname.match(/\/epub\/.+/)
		);
	}, [location]);

	if (isLoading) {
		return null;
	} else if (error?.response?.status === 401) {
		client.invalidateQueries('getLibraries');
		return <Navigate to="/auth/login" />;
	}

	return (
		<Flex w="full" h="full">
			{!hideSidebar && <Sidebar />}
			<Box as="main" w="full" h="full" bg={useColorModeValue('gray.50', 'gray.900')}>
				{!hideSidebar && <Topbar />}
				<React.Suspense fallback={<Lazy />}>
					<Outlet />
				</React.Suspense>
			</Box>
		</Flex>
	);
}
