pipeline {
    agent { label 'built-in' }

    environment {
        GITHUB_PAT = credentials('github-pat')
    }

    stages {
        stage('Deploy Rust API to K8s') {
            steps {
                echo "Deploying to Kubernetes..."
                withCredentials([sshUserPrivateKey(
                    credentialsId: 'k8s-ssh',
                    keyFileVariable: 'SSH_KEY',
                    usernameVariable: 'SSH_USER'
                )]) {
                    bat """
                        icacls %SSH_KEY% /inheritance:r
                        icacls %SSH_KEY% /grant:r "%USERNAME%:R"
                        ssh -i %SSH_KEY% -o StrictHostKeyChecking=no %SSH_USER%@192.168.56.10 "GITHUB_PAT=%GITHUB_PAT% bash /home/willow/deploy-rust-dev.sh"
                    """
                }
            }
        }
    }
}