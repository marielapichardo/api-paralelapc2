pipeline {
    agent any

    environment {

        // Variables de entorno para Nexus 
           DOCKER_REGISTRY = "10.0.0.8:8082"  
           NEXUS_REPO      = "repositorio-nexus"
           DOCKER_IMAGE    = "api-paralela"
           DOCKER_TAG      = "latest"
        
            NEXUS_USER     = "admin"
            NEXUS_PASSPRASE = "Mari0203"

           // Parámetros para AWS
           AWS_REGION      = "us-east-1"  
           ECS_CLUSTER     = "paralelacluster"  
           ECS_SERVICE     = "paralela-service"     
           TASK_FAMILY     = "paralelatask"  
           EXECUTION_ROLE  = "arn:aws:iam::831926602540:role/ecsTaskExecutionRole" 
    }

       stages {

           stage('Checkout') {
               steps {
                   git branch: 'develop', url: 'https://github.com/marielapichardo/api-paralelapc2.git'
               }
           }

           stage('Build Docker Image') {
               steps {
                   script {
                       // Construir la
                       bat "docker build -t ${DOCKER_IMAGE}:${DOCKER_TAG} ."
                       // Etiquetar la imagen para el registro
                       bat "docker tag ${DOCKER_IMAGE}:${DOCKER_TAG} ${DOCKER_REGISTRY}/${NEXUS_REPO}/${DOCKER_IMAGE}:${DOCKER_TAG}"
                   }
               }
           }

           stage('Login en Nexus') {
                steps {
                    bat "docker login ${DOCKER_REGISTRY} -u ${NEXUS_USER} -p ${NEXUS_PASSPRASE}"
                }
            }
            stage('Push en Nexus') {
                steps {
                    script {
                        bat "docker push ${DOCKER_REGISTRY}/${NEXUS_REPO}/${DOCKER_IMAGE}:${DOCKER_TAG}"
                    }
                }
            }

                   stage('Deploy servidor') {
               steps {
                   
                   withCredentials([[
                       $class: 'AmazonWebServicesCredentialsBinding',
                       credentialsId: '831926602540'
                   ]]) {
                       script {
                           // Generar el JSON de la nueva definición de tarea
                           def taskDefJson = """
                           {
                             "family": "${TASK_FAMILY}",
                             "networkMode": "awsvpc",
                             "executionRoleArn": "${EXECUTION_ROLE}",
                             "containerDefinitions": [
                               {
                                 "name": "${DOCKER_IMAGE}",
                                 "image": "${DOCKER_REGISTRY}/${NEXUS_REPO}/${DOCKER_IMAGE}:${DOCKER_TAG}",
                                 "essential": true,
                                 "portMappings": [
                                   {
                                     "containerPort": 3030,
                                     "hostPort": 3030,
                                     "protocol": "tcp"
                                   }
                                 ]
                               }
                             ],
                             "requiresCompatibilities": [
                                 "FARGATE"
                             ],
                             "cpu": "256",
                             "memory": "512"
                           }
                           """
                           // Escribir el archivo de definición de tarea
                           writeFile file: 'taskdef.json', text: taskDefJson

                           // Registrar la nueva revisión de la definición de tarea
                           bat "aws ecs register-task-definition --region ${AWS_REGION} --cli-input-json file://taskdef.json"

                           // Actualizar el servicio para forzar nueva implementación
                           bat "aws ecs update-service --region ${AWS_REGION} --cluster ${ECS_CLUSTER} --service ${ECS_SERVICE} --force-new-deployment"
                       }
                   }
               }
           }

        }
           
       post {
           success {
               echo 'Pipeline completado exitosamente y el despliegue en AWS Fargate se ha iniciado.'
           }
           failure {
               echo 'El pipeline ha fallado.'
           }
       }
}
