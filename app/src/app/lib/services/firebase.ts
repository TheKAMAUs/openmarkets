// services/firebaseUpload.ts
import { ref, uploadBytes, getDownloadURL } from 'firebase/storage';
import { storage } from '../../lib/fireConfig';

// Compression function (from above)
export const compressImage = (file: File, maxSizeMB: number = 0.5, maxDimension: number = 1024): Promise<File> => {
    return new Promise((resolve, reject) => {
        if (!file.type.startsWith('image/')) {
            resolve(file);
            return;
        }

        const reader = new FileReader();
        reader.readAsDataURL(file);

        reader.onload = (event) => {
            const img = new Image();
            img.src = event.target?.result as string;

            img.onload = () => {
                // Calculate dimensions
                let { width, height } = img;
                if (width > height && width > maxDimension) {
                    height = Math.round((height * maxDimension) / width);
                    width = maxDimension;
                } else if (height > maxDimension) {
                    width = Math.round((width * maxDimension) / height);
                    height = maxDimension;
                }

                // Draw to canvas
                const canvas = document.createElement('canvas');
                canvas.width = width;
                canvas.height = height;
                const ctx = canvas.getContext('2d');
                if (!ctx) {
                    reject(new Error('Canvas context not available'));
                    return;
                }

                ctx.drawImage(img, 0, 0, width, height);

                // Convert to blob with quality adjustment
                const tryCompress = (quality: number): Promise<File> => {
                    return new Promise((res) => {
                        canvas.toBlob((blob) => {
                            if (!blob) {
                                res(file);
                                return;
                            }

                            const targetSize = maxSizeMB * 1024 * 1024;
                            if (blob.size > targetSize && quality > 0.3) {
                                tryCompress(quality - 0.15).then(res);
                            } else {
                                const newFile = new File(
                                    [blob],
                                    file.name.replace(/\.[^/.]+$/, '.jpg'),
                                    { type: 'image/jpeg' }
                                );
                                console.log(`📦 Compressed: ${(file.size / 1024).toFixed(0)}KB → ${(blob.size / 1024).toFixed(0)}KB (${Math.round(blob.size / file.size * 100)}%)`);
                                res(newFile);
                            }
                        }, 'image/jpeg', quality);
                    });
                };

                tryCompress(0.8).then(resolve);
            };

            img.onerror = () => reject(new Error('Failed to load image'));
        };

        reader.onerror = () => reject(new Error('Failed to read file'));
    });
};

// Main upload function
export const uploadFileToFirebase = async (
    file: File,
    userId: string | undefined,
    email: string,
    documentType: string
): Promise<string> => {
    try {
        console.log(`📤 Uploading: ${file.name} (${(file.size / 1024).toFixed(0)}KB)`);

        // Compress image
        const compressedFile = await compressImage(file, 0.1, 800); // 100KB max, 800px max

        // Create safe filename
        const safeEmail = email.replace(/[@.]/g, '_');
        const ext = compressedFile.type === 'image/jpeg' ? 'jpg' : (file.name.split('.').pop() || 'bin');
        const timestamp = Date.now();
        const fileName = `${safeEmail}_${timestamp}_${documentType}.${ext}`;

        // Determine path based on whether userId exists and document type
        let filePath: string;
        let metadata: any;

        // Check if this is a market logo (documentType includes 'market_logo' or similar)
        const isMarketLogo = documentType.includes('market_logo') ||
            documentType.includes('logo') ||
            documentType === 'market_logo';

        if (!userId || isMarketLogo) {
            // For market logos or when userId is missing, store in admin/markets folder
            filePath = `admin/markets/${documentType}/${fileName}`;
            metadata = {
                contentType: compressedFile.type,
                customMetadata: {
                    uploadedBy: email || 'anonymous',
                    documentType,
                    originalName: file.name,
                    originalSize: file.size.toString(),
                    compressedSize: compressedFile.size.toString(),
                    uploadedAt: new Date().toISOString(),
                    isMarketLogo: 'true'
                }
            };
            console.log(`📁 Uploading market logo to: ${filePath}`);
        } else {
            // For user verification documents
            filePath = `users/${userId}/verification/${documentType}/${fileName}`;
            metadata = {
                contentType: compressedFile.type,
                customMetadata: {
                    userId,
                    email,
                    documentType,
                    originalName: file.name,
                    originalSize: file.size.toString(),
                    compressedSize: compressedFile.size.toString(),
                    uploadedAt: new Date().toISOString()
                }
            };
            console.log(`📁 Uploading user document to: ${filePath}`);
        }

        const storageRef = ref(storage, filePath);
        await uploadBytes(storageRef, compressedFile, metadata);

        // Get download URL
        const downloadUrl = await getDownloadURL(storageRef);

        console.log(`✅ Upload complete: ${downloadUrl}`);
        return downloadUrl;

    } catch (error) {
        console.error('❌ Upload failed:', error);
        throw error;
    }
};





// // Main upload function
// export const uploadFileToFirebase = async (
//     file: File,
//     userId: string,
//     email: string,
//     documentType: string
// ): Promise<string> => {
//     try {
//         console.log(`📤 Uploading: ${file.name} (${(file.size / 1024).toFixed(0)}KB)`);

//         // Compress image
//         const compressedFile = await compressImage(file, 0.1, 800); // 300KB max, 800px max

//         // Create safe filename
//         const safeEmail = email.replace(/[@.]/g, '_');
//         const ext = compressedFile.type === 'image/jpeg' ? 'jpg' : (file.name.split('.').pop() || 'bin');
//         const timestamp = Date.now();
//         const fileName = `${safeEmail}_${timestamp}_${documentType}.${ext}`;

//         // Firebase path
//         const filePath = `users/${userId}/verification/${documentType}/${fileName}`;
//         const storageRef = ref(storage, filePath);

//         console.log(`📁 Uploading to: ${filePath}`);

//         // Upload with metadata
//         const metadata = {
//             contentType: compressedFile.type,
//             customMetadata: {
//                 userId,
//                 email,
//                 documentType,
//                 originalName: file.name,
//                 originalSize: file.size.toString(),
//                 compressedSize: compressedFile.size.toString(),
//                 uploadedAt: new Date().toISOString()
//             }
//         };

//         await uploadBytes(storageRef, compressedFile, metadata);

//         // Get download URL
//         const downloadUrl = await getDownloadURL(storageRef);

//         console.log(`✅ Upload complete: ${downloadUrl}`);
//         return downloadUrl;

//     } catch (error) {
//         console.error('❌ Upload failed:', error);
//         throw error;
//     }
// };





