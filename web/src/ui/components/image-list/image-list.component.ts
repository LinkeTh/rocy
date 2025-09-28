import {HttpClient} from '@angular/common/http';
import {ChangeDetectionStrategy, Component, inject, OnInit, signal} from '@angular/core';
import {JsonViewModule} from 'nxt-json-view';
import {environment} from '../../../environments/environment';

export interface ImageEntity {
    id: number; // ImageId newtype (i32)
    user_id: number; // UserId newtype (i32)
    file_name: string;
    content_type: string;
    data_base64: string; // base64 encoded image
}

@Component({
    selector: 'app-image-list',
    standalone: true,
    imports: [
        JsonViewModule
    ],
    changeDetection: ChangeDetectionStrategy.OnPush,
    templateUrl: './image-list.component.html'
})
export class ImageListComponent implements OnInit {
    private readonly http = inject(HttpClient);

    readonly loading = signal(false);
    readonly error = signal<string | null>(null);
    readonly images = signal<ImageEntity[] | null>(null);

    ngOnInit(): void {
        this.refresh();
    }

    refresh() {
        if (this.loading()) return;
        this.loading.set(true);
        this.error.set(null);

        const url = `${environment.apiBase}/images`;
        this.http.get<ImageEntity[]>(url, {withCredentials: true}).subscribe({
            next: (res) => this.images.set(res ?? []),
            error: (err) => {
                const message = err?.error?.message || err?.message || 'Failed to load images';
                this.error.set(message);
            },
            complete: () => this.loading.set(false)
        });
    }

    protected readonly JSON = JSON;
}
