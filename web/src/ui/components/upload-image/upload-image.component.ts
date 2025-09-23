import {DecimalPipe} from '@angular/common';
import {HttpClient} from '@angular/common/http';
import {ChangeDetectionStrategy, Component, inject, output, signal} from '@angular/core';
import {environment} from '../../../environments/environment';

@Component({
    selector: 'app-upload-image',
    standalone: true,
    imports: [DecimalPipe],
    changeDetection: ChangeDetectionStrategy.OnPush,
    templateUrl: './upload-image.component.html'
})
export class UploadImageComponent {
    private http = inject(HttpClient);

    readonly selectedFile = signal<File | null>(null);
    readonly previewUrl = signal<string | null>(null);
    readonly uploading = signal(false);
    readonly error = signal<string | null>(null);
    readonly success = signal<boolean>(false);

    readonly uploaded = output<unknown>();

    onFileChange(event: Event) {
        this.error.set(null);
        const input = event.target as HTMLInputElement;
        const file = input.files && input.files[0] ? input.files[0] : null;

        if (file && !file.type.startsWith('image/')) {
            this.error.set('Only image files are allowed.');
            this.clearSelection();
            return;
        }

        this.selectedFile.set(file);

        // Create preview
        if (file) {
            const reader = new FileReader();
            reader.onload = () => this.previewUrl.set(reader.result as string);
            reader.readAsDataURL(file);
        } else {
            this.previewUrl.set(null);
        }
    }

    cancel() {
        this.clearSelection();
        this.success.set(false);
        this.error.set(null);
    }

    save() {
        const file = this.selectedFile();
        if (!file || this.uploading()) return;

        const form = new FormData();
        form.append('file', file, file.name);

        this.uploading.set(true);
        this.error.set(null);
        this.success.set(false);

        const url = `${environment.apiBase}/upload`;
        this.http.post(url, form, {withCredentials: true}).subscribe({
            next: (res) => {
                this.success.set(true);
                this.uploaded.emit(res);
                // keep the selection to allow user to see what was uploaded; they can cancel to reset
            },
            error: (err) => {
                const message = err?.error?.message || err?.message || 'Upload failed';
                this.error.set(message);
            },
            complete: () => this.uploading.set(false)
        });
    }

    private clearSelection() {
        this.selectedFile.set(null);
        this.previewUrl.set(null);
    }
}
