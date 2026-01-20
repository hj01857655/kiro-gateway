import { Component, ErrorInfo, ReactNode } from 'react';
import { Card, Title, Text, Button, Stack, Center, rem } from '@mantine/core';
import { AlertCircle, RefreshCw } from 'lucide-react';

interface Props {
    children?: ReactNode;
}

interface State {
    hasError: boolean;
    error: Error | null;
}

class ErrorBoundary extends Component<Props, State> {
    public state: State = {
        hasError: false,
        error: null,
    };

    public static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error };
    }

    public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error('Uncaught error:', error, errorInfo);
    }

    private handleReset = () => {
        this.setState({ hasError: false, error: null });
        window.location.reload();
    };

    public render() {
        if (this.state.hasError) {
            return (
                <Center h="100vh" p="md">
                    <Card withBorder padding="xl" radius="md" className="glass-effect" style={{ maxWidth: rem(500), textAlign: 'center' }}>
                        <Stack align="center" gap="md">
                            <div
                                style={{
                                    width: rem(64),
                                    height: rem(64),
                                    borderRadius: '50%',
                                    background: 'rgba(250, 82, 82, 0.1)',
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                }}
                            >
                                <AlertCircle size={32} color="#fa5252" />
                            </div>
                            <Title order={2}>糟糕，出错了</Title>
                            <Text c="dimmed" size="sm">
                                应用程序遇到了预期之外的错误。请尝试刷新页面或点击下方按钮重试。
                            </Text>
                            {this.state.error && (
                                <Text size="xs" c="dimmed" style={{
                                    background: 'rgba(0,0,0,0.05)',
                                    padding: rem(8),
                                    borderRadius: rem(4),
                                    fontFamily: 'monospace',
                                    width: '100%',
                                    textAlign: 'left',
                                    maxHeight: rem(150),
                                    overflowY: 'auto'
                                }}>
                                    {this.state.error.message}
                                </Text>
                            )}
                            <Button
                                leftSection={<RefreshCw size={16} />}
                                onClick={this.handleReset}
                                variant="filled"
                                color="indigo"
                                fullWidth
                            >
                                重启应用
                            </Button>
                        </Stack>
                    </Card>
                </Center>
            );
        }

        return this.props.children;
    }
}

export default ErrorBoundary;
