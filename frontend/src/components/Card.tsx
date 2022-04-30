import { Box, Text } from '@chakra-ui/react';
import React from 'react';
import { Link } from 'react-router-dom';

export interface CardProps {
	to: string;
	imageAlt: string;
	imageSrc: string;
	title: string;
	subtitle?: string;
	onMouseEnter?: () => void;
}

export default function Card({ to, imageAlt, imageSrc, title, subtitle, onMouseEnter }: CardProps) {
	return (
		<Box
			as={Link}
			shadow="base"
			bg="gray.50"
			border="1.5px solid"
			borderColor="transparent"
			_dark={{ bg: 'gray.750' }}
			_hover={{
				borderColor: 'brand.500',
			}}
			rounded="md"
			to={to}
			onMouseEnter={onMouseEnter}
		>
			<Box px={1.5}>
				<img
					alt={imageAlt}
					className="h-72 w-[12rem] object-cover"
					src={imageSrc}
					onError={(err) => {
						// @ts-ignore
						err.target.src = '/src/favicon.png';
					}}
				/>
			</Box>

			<Box className="max-w-[11.5rem] p-2" color="black" _dark={{ color: 'gray.100' }}>
				<Text size="sm" as="h3">
					{title}
				</Text>

				<Text size="sm">{subtitle}</Text>
			</Box>
		</Box>
	);
}