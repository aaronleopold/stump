import { ButtonProps as ChakraButtonProps, Button as ChakraButton } from '@chakra-ui/react';
import React from 'react';

export interface ButtonProps extends ChakraButtonProps {}

export default function Button(props: ButtonProps) {
	return (
		<ChakraButton
			{...props}
			_focus={{
				boxShadow: '0 0 0 2px rgba(196, 130, 89, 0.6);',
			}}
		/>
	);
}
